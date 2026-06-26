import { useState } from "react";

import { callApi } from "../api";
import type { AnswerValue, AppView, QuestionView } from "../types";
import { Check, ConsequenceCard, VerdictBanner } from "./visuals";

export default function Wizard({
	view,
	setView,
}: {
	view: AppView;
	setView: (v: AppView) => void;
}) {
	const [busy, setBusy] = useState(false);

	const answers = view.answers as unknown as Record<string, AnswerValue>;
	const ev = view.evaluation;
	const byId = new Map(view.questions.map((q) => [q.id, q]));
	const offDefault = ev.consequences.filter(
		(c) => c.consequence.severity === "NonDefault",
	).length;
	const blocking = ev.consequences.filter((c) => c.consequence.severity === "Blocking").length;
	const guidanceFor = (qid: string) => ev.guidance.find((g) => g.at === qid)?.message;

	async function change(qid: string, value: AnswerValue) {
		const next = { ...answers, [qid]: value };
		// Reflect the choice immediately; the server's re-evaluation follows.
		setView({ ...view, answers: next as unknown as AppView["answers"] });
		setBusy(true);
		try {
			setView(await callApi("applications", "patch", { id: view.id, answers: next }));
		} finally {
			setBusy(false);
		}
	}

	async function finalize() {
		setBusy(true);
		try {
			setView(await callApi("applications", "finalize", { id: view.id }));
		} finally {
			setBusy(false);
		}
	}

	return (
		<div className="frame">
			<aside className="rail">
				<VerdictBanner verdict={ev.verdict} offDefault={offDefault} blocking={blocking} />
				<div className="meters">
					<div className="meter">
						<span className="meter-k">Size</span>
						<span className="meter-v">{ev.derived["size"] ?? "—"}</span>
					</div>
					<div className="meter">
						<span className="meter-k">Off-default</span>
						<span className="meter-v" style={{ color: offDefault ? "var(--offdef)" : undefined }}>
							{offDefault}
						</span>
					</div>
					<div className="meter">
						<span className="meter-k">Blocking</span>
						<span className="meter-v" style={{ color: blocking ? "var(--block)" : undefined }}>
							{blocking}
						</span>
					</div>
				</div>
				<div className="rail-section-h">Consequences</div>
				<div className="ledger">
					{ev.consequences.length === 0 ? (
						<p className="ledger-empty">
							Choices that leave the default path show up here as you go.
						</p>
					) : (
						ev.consequences.map((c) => <ConsequenceCard key={c.id} c={c.consequence} />)
					)}
				</div>
			</aside>

			<main className="main">
				{ev.visible_questions.map((qid) => {
					const q = byId.get(qid);
					return q ? (
						<QuestionCard
							key={q.id}
							q={q}
							value={answers[q.id]}
							guidance={guidanceFor(q.id)}
							onChange={(v) => change(q.id, v)}
						/>
					) : null;
				})}
				<div className="actions">
					<button type="button" className="btn primary" disabled={busy} onClick={finalize}>
						Finalize
					</button>
				</div>
			</main>
		</div>
	);
}

function QuestionCard({
	q,
	value,
	guidance,
	onChange,
}: {
	q: QuestionView;
	value: AnswerValue | undefined;
	guidance: string | undefined;
	onChange: (v: AnswerValue) => void;
}) {
	return (
		<div className="card">
			<h3 className="qtitle">{q.label}</h3>
			{q.help && <p className="qhelp">{q.help}</p>}
			{guidance && <div className="guide">{guidance}</div>}

			{q.kind === "Band" ? (
				<div className="bandrow">
					{q.options.map((o) => (
						<button
							type="button"
							key={o.id}
							className={`band${value === o.id ? " on" : ""}`}
							onClick={() => onChange(o.id)}
						>
							{o.label}
						</button>
					))}
				</div>
			) : (
				<div className="choices">
					{q.options.map((o) => {
						const selected =
							q.kind === "Multi"
								? Array.isArray(value) && value.includes(o.id)
								: value === o.id;
						return (
							<button
								type="button"
								key={o.id}
								className={`choice${selected ? " on" : ""}`}
								onClick={() => onChange(q.kind === "Multi" ? toggle(value, o.id) : o.id)}
							>
								<span className="choice-tick">{selected && <Check size={13} />}</span>
								<span>
									<span className="choice-title">{o.label}</span>
									{o.note && <span className="choice-note">{o.note}</span>}
								</span>
							</button>
						);
					})}
				</div>
			)}
		</div>
	);
}

function toggle(value: AnswerValue | undefined, id: string): string[] {
	const arr = Array.isArray(value) ? value : [];
	return arr.includes(id) ? arr.filter((x) => x !== id) : [...arr, id];
}
