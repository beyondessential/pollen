import { useState } from "react";
import { useNavigate } from "react-router-dom";

import { callApi } from "../api";
import {
	type AnswerValue,
	type AppView,
	AUDIENCE_LABEL,
	type Audience,
	type QuestionView,
} from "../types";
import { ConsequenceCard, VerdictBanner } from "./visuals";

const AUDIENCE_ORDER: Audience[] = ["Client", "Bes", "Record"];

export default function Artifact({ view }: { view: AppView }) {
	const navigate = useNavigate();
	const [busy, setBusy] = useState(false);
	const [copied, setCopied] = useState(false);

	const answers = view.answers as unknown as Record<string, AnswerValue>;
	const ev = view.evaluation;
	const offDefault = ev.consequences.filter(
		(c) => c.consequence.severity === "NonDefault",
	).length;
	const blocking = ev.consequences.filter((c) => c.consequence.severity === "Blocking").length;
	const started = Object.values(answers).some((v) =>
		Array.isArray(v) ? v.length > 0 : v != null && v !== "",
	);

	async function makeNewVersion() {
		setBusy(true);
		try {
			const forked = await callApi("applications", "fork", { id: view.id });
			navigate(`/a/${forked.id}`);
		} finally {
			setBusy(false);
		}
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

	const created = view.created_at.slice(0, 10);

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
					<div className="mono">{created}</div>
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

			<div className="sheet-actions">
				<button type="button" className="btn ghost" onClick={copyLink}>
					{copied ? "Link copied" : "Copy link"}
				</button>
				<button type="button" className="btn ghost" disabled={busy} onClick={makeNewVersion}>
					Make changes (new version)
				</button>
			</div>

			{AUDIENCE_ORDER.map((audience) => {
				const items = ev.consequences.filter((c) => c.consequence.audience === audience);
				return items.length ? (
					<section key={audience} className="sheet-section">
						<h3 className="sheet-section-title">{AUDIENCE_LABEL[audience]}</h3>
						{items.map((c) => (
							<ConsequenceCard key={c.id} c={c.consequence} />
						))}
					</section>
				) : null;
			})}

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

function answerLabel(q: QuestionView, value: AnswerValue | undefined): string {
	if (value == null || (Array.isArray(value) && value.length === 0)) return "—";
	const label = (id: string) => q.options.find((o) => o.id === id)?.label ?? id;
	return Array.isArray(value) ? value.map(label).join(", ") : label(value);
}

function topology(questions: QuestionView[], answers: Record<string, AnswerValue>): string {
	const central = answerLabel(byId(questions, "central"), answers["central"]);
	const mixQ = byId(questions, "facility_mix");
	const mix = answerLabel(mixQ, answers["facility_mix"]);
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
	return questions.find((q) => q.id === id) ?? { id, kind: "Single", label: id, help: null, options: [] };
}
