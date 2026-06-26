import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";

import { callApi } from "../api";
import {
	type AnswerValue,
	type AppView,
	AUDIENCE_LABEL,
	type Audience,
	type QuestionView,
	TOPIC_LABEL,
	TOPIC_ORDER,
	type TriggeredConsequence,
} from "../types";
import { ConsequenceCard, VerdictBanner } from "./visuals";

const AUDIENCE_ORDER: Audience[] = ["Client", "Bes", "Record"];

type Grouping = "audience" | "topic";
type Group = { key: string; label: string; items: TriggeredConsequence[] };

export default function Artifact({ view }: { view: AppView }) {
	const navigate = useNavigate();
	const [busy, setBusy] = useState(false);
	const [copied, setCopied] = useState(false);
	const [grouping, setGrouping] = useState<Grouping>("audience");
	const [query, setQuery] = useState("");

	const answers = view.answers as unknown as Record<string, AnswerValue>;
	const ev = view.evaluation;
	const offDefault = ev.consequences.filter(
		(c) => c.consequence.severity === "NonDefault",
	).length;
	const blocking = ev.consequences.filter((c) => c.consequence.severity === "Blocking").length;
	const started = Object.values(answers).some((v) =>
		Array.isArray(v) ? v.length > 0 : v != null && v !== "",
	);

	const groups = useMemo(() => {
		const q = query.trim().toLowerCase();
		const matches = q
			? ev.consequences.filter((c) =>
					`${c.consequence.title} ${c.consequence.detail}`.toLowerCase().includes(q),
				)
			: ev.consequences;
		return groupConsequences(matches, grouping).filter((g) => g.items.length > 0);
	}, [ev.consequences, grouping, query]);

	async function makeNewVersion() {
		setBusy(true);
		try {
			const forked = await callApi("applications", "fork", { id: view.id });
			navigate(`/a/${forked.id}`);
		} finally {
			setBusy(false);
		}
	}

	function downloadPdf() {
		// Print the complete, audience-sectioned artifact regardless of the
		// current toggle/search; let React re-render before the print dialog.
		setGrouping("audience");
		setQuery("");
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
					<h2 className="sheet-title">{ev.derived["size"] ?? "Unsized"} deployment</h2>
					<div className="sheet-facts">
						<span>{topology(view.questions, answers)}</span>
						{regionLabel(view.questions, answers) && (
							<span>Region: {regionLabel(view.questions, answers)}</span>
						)}
					</div>
				</div>
				<div className="sheet-meta">
					<div className="mono">config {view.config_hash.slice(0, 12)}</div>
					<div className="mono">{view.created_at.slice(0, 10)}</div>
				</div>
			</div>

			<div style={{ padding: "22px 30px 0" }}>
				<VerdictBanner
					verdict={ev.verdict}
					offDefault={offDefault}
					blocking={blocking}
					started={started}
					big
				/>
			</div>

			<div className="sheet-controls">
				<div className="tg-group">
					<button
						type="button"
						className={`tg${grouping === "audience" ? " on" : ""}`}
						onClick={() => setGrouping("audience")}
					>
						By audience
					</button>
					<button
						type="button"
						className={`tg${grouping === "topic" ? " on" : ""}`}
						onClick={() => setGrouping("topic")}
					>
						By topic
					</button>
				</div>
				<input
					className="sheet-search"
					type="search"
					placeholder="Search consequences…"
					value={query}
					onChange={(e) => setQuery(e.target.value)}
				/>
				<div className="sheet-control-actions">
					<button type="button" className="btn ghost" onClick={copyLink}>
						{copied ? "Link copied" : "Copy link"}
					</button>
					<button type="button" className="btn ghost" onClick={downloadPdf}>
						Download PDF
					</button>
					<button type="button" className="btn ghost" disabled={busy} onClick={makeNewVersion}>
						Make changes
					</button>
				</div>
			</div>

			{groups.length === 0 ? (
				<section className="sheet-section">
					<p className="ledger-empty">No consequences match your search.</p>
				</section>
			) : (
				groups.map((g) => (
					<section key={g.key} id={`s-${g.key}`} className="sheet-section">
						<h3 className="sheet-section-title">{g.label}</h3>
						{g.items.map((c) => (
							<ConsequenceCard key={c.id} c={c.consequence} />
						))}
					</section>
				))
			)}

			<section className="sheet-section">
				<h3 className="sheet-section-title">Full decision record</h3>
				<div className="record">
					{view.questions.map((q) => (
						<div className="record-row" key={q.id}>
							<span>{q.label}</span>
							<span className="mono">{answerLabel(q, answers[q.id])}</span>
						</div>
					))}
				</div>
			</section>
		</div>
	);
}

function groupConsequences(items: TriggeredConsequence[], grouping: Grouping): Group[] {
	if (grouping === "audience") {
		return AUDIENCE_ORDER.map((a) => ({
			key: a,
			label: AUDIENCE_LABEL[a],
			items: items.filter((c) => c.consequence.audience === a),
		}));
	}
	// By topic: known topics first, then any others in order of appearance.
	const extras = items.map((c) => c.source).filter((s) => !TOPIC_ORDER.includes(s));
	const sources = [...TOPIC_ORDER, ...new Set(extras)];
	return sources.map((source) => ({
		key: source,
		label: TOPIC_LABEL[source] ?? source,
		items: items.filter((c) => c.source === source),
	}));
}

function answerLabel(q: QuestionView, value: AnswerValue | undefined): string {
	if (value == null || (Array.isArray(value) && value.length === 0)) return "—";
	const label = (id: string) => q.options.find((o) => o.id === id)?.label ?? id;
	return Array.isArray(value) ? value.map(label).join(", ") : label(value);
}

function topology(questions: QuestionView[], answers: Record<string, AnswerValue>): string {
	const central = answerLabel(byId(questions, "central"), answers["central"]);
	const mix = answerLabel(byId(questions, "facility_mix"), answers["facility_mix"]);
	return `Central: ${central} · Facilities: ${mix}`;
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
		}
	);
}
