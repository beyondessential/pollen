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
import { Markup } from "../markup";

// Warnings lead the sheet, so what the client is opting into is met before
// anything else, but collapsed: they are context rather than a task. They carry
// the off-default colour they carry everywhere else.
const AUDIENCE_ORDER: Audience[] = ["Record", "Client", "Bes", "Pricing"];
const COLLAPSED_ON_ARRIVAL: Audience[] = ["Record"];
const WARNINGS: Audience = "Record";

type Group = { key: string; label: string; items: TriggeredConsequence[] };

export default function Artifact({ view }: { view: AppView }) {
	const navigate = useNavigate();
	const [busy, setBusy] = useState(false);
	const [copied, setCopied] = useState(false);
	// Groups open on arrival, bar the acknowledgements: those restate choices the
	// reader has just made, and the viability callout raises anything that will
	// not work regardless.
	const [shut, setShut] = useState<Record<string, boolean>>(() =>
		Object.fromEntries(COLLAPSED_ON_ARRIVAL.map((a) => [a, true])),
	);
	// Ticked-off actions, this reader's own and kept on this device only.
	const [done, setDone] = useState<string[]>(() => listDone(view.id));

	function toggleDone(id: string) {
		const next = done.includes(id) ? done.filter((d) => d !== id) : [...done, id];
		setDone(next);
		recordDone(view.id, next);
	}

	const answers = view.answers as unknown as Record<string, AnswerValue>;
	const ev = view.evaluation;
	const byId = new Map(view.questions.map((q) => [q.id, q]));
	const assumedBy = new Map(ev.assumed.map((a) => [a.question, a.option]));
	// An artifact is interim when questions were deliberately left open. It is a
	// normal finalised artifact in every other respect: the gaps are recorded
	// rather than guessed, and completing it is a new version.
	const interim = ev.open_items.length > 0;
	// Conflicts that make the configuration unworkable, for the viability callout.
	const conflicts = ev.consequences
		.filter((c) => c.consequence.severity === "Blocking")
		.map((c) => c.consequence.title);

	const groups = useMemo(
		() => groupConsequences(ev.consequences).filter((g) => g.items.length > 0),
		[ev.consequences],
	);
	// Warnings lead the sheet: what the client is opting into is met before
	// anything else. The rest are the work, and sit under "Next steps".
	const warningGroup = groups.find((g) => g.key === WARNINGS);
	const actionGroups = groups.filter((g) => g.key !== WARNINGS);

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
		// Print the whole artifact regardless of what is collapsed; let React
		// re-render before the print dialog opens.
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

	function renderGroup(g: Group) {
		const open = !shut[g.key];
		const warn = g.key === WARNINGS;
		return (
			<section
				key={g.key}
				id={`s-${g.key}`}
				className={`sheet-section${warn ? " sheet-section-flush" : ""}`}
			>
				<button
					type="button"
					className={`qexpand${warn ? " qexpand-warn" : ""}${open ? " on" : ""}`}
					aria-expanded={open}
					onClick={() => setShut({ ...shut, [g.key]: open })}
				>
					<Chevron size={17} />
					<span>{g.label}</span>
					<span className="group-count">{g.items.length}</span>
				</button>
				{/* Always rendered, hidden with CSS: printing must carry the whole
				    record however the reader reached the print dialog. */}
				<div className={`items${open ? "" : " shut"}`}>
					{g.items.map((c) => (
						<ConsequenceCard
							key={c.id}
							c={c.consequence}
							done={done.includes(c.id)}
							onToggle={warn ? undefined : () => toggleDone(c.id)}
						/>
					))}
				</div>
			</section>
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
						<span>{topology(view.questions, answers, assumedBy)}</span>
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

			{conflicts.length > 0 && (
				<div style={{ padding: "22px 30px 0" }}>
					<VerdictBanner conflicts={conflicts} />
				</div>
			)}

			{warningGroup && renderGroup(warningGroup)}

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

			{ev.requirements.length > 0 && (
				<section className="sheet-section" id="s-compute">
					<h3 className="sheet-section-title">Compute requirements</h3>
					<p className="sheet-caption">What this deployment needs, sized from your answers.</p>
					<div className="reqs">
						{ev.requirements.map((r) => (
							<article className="req" key={r.id}>
								<header className="req-head">
									<h4 className="req-class">{r.class}</h4>
									{r.summary && (
										<p className="req-summary">
											<Markup text={r.summary} />
										</p>
									)}
								</header>
								<dl className="req-specs">
									{r.specs.map((s) => (
										<div className="req-spec" key={s.label}>
											<dt>{s.label}</dt>
											<dd>{s.value}</dd>
										</div>
									))}
								</dl>
								{r.notes.length > 0 && (
									<div className="req-notes">
										{r.notes.map((n) => (
											<p className="req-note" key={n}>
												<Markup text={n} />
											</p>
										))}
									</div>
								)}
							</article>
						))}
					</div>
				</section>
			)}



			{actionGroups.length > 0 && (
				<div className="sheet-heading">
					<h3 className="sheet-section-title">Next steps</h3>
					<p className="sheet-caption">Who needs to do what to stand this deployment up.</p>
				</div>
			)}

			{groups.length === 0 && (
				<section className="sheet-section">
					<p className="ledger-empty">No consequences match your search.</p>
				</section>
			)}
			{actionGroups.map(renderGroup)}

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

function topology(
	questions: QuestionView[],
	answers: Record<string, AnswerValue>,
	assumed: Map<string, string>,
): string {
	// An assumed answer is the answer until the reader changes it, so the header
	// reads it the same way the engine does.
	const fact = (id: string) => {
		const q = byId(questions, id);
		const value = isAnswered(answers[id]) ? answers[id] : assumed.get(id);
		return value ? answerLabel(q, value) : "Not answered";
	};
	return `Central: ${fact("central")} · Facilities: ${fact("hosting_where")}`;
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
