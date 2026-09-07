import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";

import { callApi } from "../api";
import {
	type AnswerValue,
	type AppView,
	AUDIENCE_LABEL,
	type Audience,
	isAnswered,
	type QuestionView,
	type TriggeredConsequence,
} from "../types";
import { listDone, recordDone } from "../doneItems";
import { Chevron, ConsequenceCard, VerdictBanner } from "./visuals";

const AUDIENCE_ORDER: Audience[] = ["Client", "Bes", "Record"];

type Group = { key: string; label: string; items: TriggeredConsequence[] };

export default function Artifact({ view }: { view: AppView }) {
	const navigate = useNavigate();
	const [busy, setBusy] = useState(false);
	const [copied, setCopied] = useState(false);
	const [query, setQuery] = useState("");
	// Groups open on arrival; collapsing is for skipping past the ones addressed
	// to someone else, never for hiding content from a reader who has not looked.
	const [shut, setShut] = useState<Record<string, boolean>>({});
	// Ticked-off actions, this reader's own and kept on this device only.
	const [done, setDone] = useState<string[]>(() => listDone(view.id));

	function toggleDone(id: string) {
		const next = done.includes(id) ? done.filter((d) => d !== id) : [...done, id];
		setDone(next);
		recordDone(view.id, next);
	}

	const answers = view.answers as unknown as Record<string, AnswerValue>;
	const ev = view.evaluation;
	const offDefault = ev.consequences.filter(
		(c) => c.consequence.severity === "NonDefault",
	).length;
	const blocking = ev.consequences.filter((c) => c.consequence.severity === "Blocking").length;
	const byId = new Map(view.questions.map((q) => [q.id, q]));
	const assumedBy = new Map(ev.assumed.map((a) => [a.question, a.option]));
	// An artifact is interim when questions were deliberately left open. It is a
	// normal finalised artifact in every other respect: the gaps are recorded
	// rather than guessed, and completing it is a new version.
	const interim = ev.open_items.length > 0;
	// What took the plan off the standard path, worst first, for the verdict.
	const rank = { Blocking: 0, NonDefault: 1, Default: 2 };
	const offStandard = ev.consequences
		.filter((c) => c.consequence.severity !== "Default")
		.sort((a, b) => rank[a.consequence.severity] - rank[b.consequence.severity])
		.map((c) => c.consequence.title);

	const groups = useMemo(() => {
		const q = query.trim().toLowerCase();
		const matches = q
			? ev.consequences.filter((c) =>
					`${c.consequence.title} ${c.consequence.detail}`.toLowerCase().includes(q),
				)
			: ev.consequences;
		return groupConsequences(matches).filter((g) => g.items.length > 0);
	}, [ev.consequences, query]);

	async function makeNewVersion() {
		// Open the tab synchronously within the click so it isn't popup-blocked,
		// then point it at the fork once created (fall back to in-place if the
		// browser blocked the window anyway).
		const tab = window.open("about:blank", "_blank");
		setBusy(true);
		try {
			const forked = await callApi("applications", "fork", { id: view.id });
			if (tab) tab.location.href = `/a/${forked.id}`;
			else navigate(`/a/${forked.id}`);
		} finally {
			setBusy(false);
		}
	}

	function downloadPdf() {
		// Print the whole artifact regardless of what is collapsed or searched
		// for; let React re-render before the print dialog opens.
		setQuery("");
		setShut({});
		setTimeout(() => window.print(), 50);
	}

	function copyLink() {
		navigator.clipboard?.writeText(window.location.href).then(
			() => {
				setCopied(true);
				setTimeout(() => setCopied(false), 1800);
			},
			() => {},
		);
	}

	return (
		<div className="sheet">
			<div className="sheet-head">
				<div>
					<h2 className="sheet-title">
						{ev.derived["size"] ?? "Unsized"} deployment
						{interim && <span className="badge-interim">Interim</span>}
					</h2>
					<div className="sheet-facts">
						<span>{topology(view.questions, answers)}</span>
						{regionLabel(view.questions, answers) && (
							<span>Region: {regionLabel(view.questions, answers)}</span>
						)}
						<span>{view.created_at.slice(0, 10)}</span>
					</div>
				</div>
				<div className="sheet-actions">
					<button type="button" className="btn ghost" onClick={copyLink}>
						{copied ? "Link copied" : "Copy link"}
					</button>
					<button type="button" className="btn ghost" onClick={downloadPdf}>
						Download PDF
					</button>
					<button type="button" className="btn ghost" disabled={busy} onClick={makeNewVersion}>
						{interim ? "Complete this plan" : "Make changes"}
					</button>
				</div>
			</div>

			{ev.verdict !== "Clear" && (
				<div style={{ padding: "22px 30px 0" }}>
					<VerdictBanner
						verdict={ev.verdict}
						offDefault={offDefault}
						blocking={blocking}
						choices={offStandard}
					/>
				</div>
			)}

			{interim && (
				<section className="sheet-section" id="s-open">
					<h3 className="sheet-section-title">To confirm with BES</h3>
					<div className="record">
						{ev.open_items.map((qid) => (
							<div className="record-row record-open" key={qid}>
								<span>{byId.get(qid)?.label ?? qid}</span>
								<span className="mono">Not yet decided</span>
							</div>
						))}
					</div>
				</section>
			)}

			{/* Searching a list you can take in at a glance is chrome, not help. A
			    plan on the standard path lands around ten, so the line sits above
			    that: search appears only for the genuinely long ones. */}
			{ev.consequences.length > 12 && (
				<div className="sheet-controls">
					<input
						className="sheet-search"
						type="search"
						placeholder="Search consequences…"
						value={query}
						onChange={(e) => setQuery(e.target.value)}
					/>
				</div>
			)}

			{groups.length === 0 ? (
				<section className="sheet-section">
					<p className="ledger-empty">No consequences match your search.</p>
				</section>
			) : (
				groups.map((g) => {
					const open = !shut[g.key];
					return (
						<section key={g.key} id={`s-${g.key}`} className="sheet-section">
							<button
								type="button"
								className={`qexpand${open ? " on" : ""}`}
								aria-expanded={open}
								onClick={() => setShut({ ...shut, [g.key]: open })}
							>
								<Chevron size={17} />
								<span>{g.label}</span>
								<span className="group-count">{g.items.length}</span>
							</button>
							{/* Always rendered, hidden with CSS: printing must carry the
							    whole record however the reader reached the print dialog. */}
							<div className={`items${open ? "" : " shut"}`}>
								{g.items.map((c) => (
									<ConsequenceCard
										key={c.id}
										c={c.consequence}
										done={done.includes(c.id)}
										onToggle={isAction(c) ? () => toggleDone(c.id) : undefined}
									/>
								))}
							</div>
						</section>
					);
				})
			)}

			{ev.assumed.length > 0 && (
				<section className="sheet-section" id="s-assumed">
					<h3 className="sheet-section-title">Assumptions</h3>
					<div className="record">
						{ev.assumed.map((a) => (
							<div className="record-row record-assumed" key={a.question}>
								<span>{byId.get(a.question)?.label ?? a.question}</span>
								<span className="mono">{optionLabel(byId.get(a.question), a.option)}</span>
							</div>
						))}
					</div>
				</section>
			)}

			<section className="sheet-section">
				<h3 className="sheet-section-title">Full decision record</h3>
				<div className="record">
					{view.questions.map((q) => (
						<div className="record-row" key={q.id}>
							<span>{q.label}</span>
							<span className="mono">
								{isAnswered(answers[q.id])
									? answerLabel(q, answers[q.id])
									: assumedBy.has(q.id)
										? optionLabel(q, assumedBy.get(q.id) ?? "")
										: "Not yet decided"}
							</span>
						</div>
					))}
				</div>
			</section>
		</div>
	);
}

/// Whether an item is something the client's IT team has to do, as opposed to
/// information or an acknowledgement of a choice. Only these get a tick box.
function isAction(c: TriggeredConsequence): boolean {
	return c.consequence.status === "Requirement" && c.consequence.audience === "Client";
}

/// Consequences grouped by the reader they are addressed to, in reading order.
function groupConsequences(items: TriggeredConsequence[]): Group[] {
	return AUDIENCE_ORDER.map((a) => ({
		key: a,
		label: AUDIENCE_LABEL[a],
		items: items.filter((c) => c.consequence.audience === a),
	}));
}

function optionLabel(q: QuestionView | undefined, id: string): string {
	return q?.options.find((o) => o.id === id)?.label ?? id;
}

function answerLabel(q: QuestionView, value: AnswerValue | undefined): string {
	if (!isAnswered(value)) return "Not answered";
	if (Array.isArray(value)) return value.map((id) => optionLabel(q, id)).join(", ");
	return optionLabel(q, value);
}

function topology(questions: QuestionView[], answers: Record<string, AnswerValue>): string {
	const central = answerLabel(byId(questions, "central"), answers["central"]);
	const where = answerLabel(byId(questions, "hosting_where"), answers["hosting_where"]);
	return `Central: ${central} · Facilities: ${where}`;
}

function regionLabel(
	questions: QuestionView[],
	answers: Record<string, AnswerValue>,
): string | null {
	const value = answers["region"];
	if (typeof value !== "string") return null;
	return answerLabel(byId(questions, "region"), value);
}

function byId(questions: QuestionView[], id: string): QuestionView {
	return (
		questions.find((q) => q.id === id) ?? {
			id,
			kind: "Single",
			label: id,
			help: null,
			options: [],
			section: null,
			default: null,
		}
	);
}
