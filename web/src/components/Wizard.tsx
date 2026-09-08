import { useMemo, useState } from "react";

import { callApi } from "../api";
import { Markup } from "../markup";
import {
	type AnswerValue,
	type AppView,
	isAnswered,
	type Opt,
	type QuestionView,
	type Section,
} from "../types";
import { Check, Chevron } from "./visuals";

export default function Wizard({
	view,
	setView,
}: {
	view: AppView;
	setView: (v: AppView) => void;
}) {
	const [busy, setBusy] = useState(false);
	// Sections the ruleset marks collapsed start closed: a non-technical user
	// can finish without ever opening them, because everything inside either
	// takes its blessed default or records as an open item.
	const [opened, setOpened] = useState<Record<string, boolean>>({});

	const answers = view.answers as unknown as Record<string, AnswerValue>;
	const ev = view.evaluation;
	const byId = new Map(view.questions.map((q) => [q.id, q]));
	const assumedBy = new Map(ev.assumed.map((a) => [a.question, a.option]));
	const guidanceFor = (qid: string) => ev.guidance.find((g) => g.at === qid)?.message;

	// The rail reports only what the user has steered off the standard path.
	// Everything else, the ordinary requirements that follow from a supported
	// setup, belongs on the finalised artifact rather than in the way of answering.
	const rank = { Blocking: 0, NonDefault: 1, Default: 2 };
	const flagged = ev.consequences
		.filter((c) => c.consequence.severity !== "Default")
		.sort((a, b) => rank[a.consequence.severity] - rank[b.consequence.severity]);

	// Group the visible questions into their sections, keeping ruleset order
	// within each. A question naming no section falls into the first one.
	const grouped = useMemo(() => {
		const sections: Section[] = view.sections.length
			? view.sections
			: [{ id: "", label: "", collapsed: false }];
		const fallback = sections[0].id;
		return sections
			.map((section) => ({
				section,
				questions: ev.visible_questions
					.map((qid) => byId.get(qid))
					.filter((q): q is QuestionView => !!q)
					.filter((q) => (q.section ?? fallback) === section.id),
			}))
			.filter((g) => g.questions.length > 0);
		// biome-ignore lint/correctness/useExhaustiveDependencies: byId is derived from view.questions
	}, [view.sections, view.questions, ev.visible_questions]);

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

	async function finalise() {
		setBusy(true);
		try {
			setView(await callApi("applications", "finalise", { id: view.id }));
		} finally {
			setBusy(false);
		}
	}

	const missing = ev.required
		.map((qid) => byId.get(qid)?.label)
		.filter((l): l is string => !!l);
	const interim = ev.open_items.length > 0;

	return (
		<div className="frame">
			<aside className="rail">
				<div className="stat">
					<span className="stat-k">Size</span>
					<span className="stat-v">{ev.derived["size"] ?? "Not sized"}</span>
				</div>

				{flagged.length > 0 && (
					<>
						<div className="rail-section-h">Off the standard path</div>
						<div className="flags">
							{flagged.map((c) => (
								<div
									key={c.id}
									className={`flag${c.consequence.severity === "Blocking" ? " flag-block" : ""}`}
								>
									<span className="flag-dot" />
									<span>{c.consequence.title}</span>
								</div>
							))}
						</div>
					</>
				)}
			</aside>

			<main className="main">
				{grouped.map(({ section, questions }) => {
					const isOpen = opened[section.id] ?? !section.collapsed;
					return (
						<section className="qsection" key={section.id || "all"}>
							{section.collapsed ? (
								<button
									type="button"
									className={`qexpand${isOpen ? " on" : ""}`}
									aria-expanded={isOpen}
									onClick={() => setOpened({ ...opened, [section.id]: !isOpen })}
								>
									<Chevron size={17} />
									<span>{section.label}</span>
								</button>
							) : (
								section.label && <h2 className="qsection-title">{section.label}</h2>
							)}
							{isOpen &&
								questions.map((q) => (
									<QuestionCard
										key={q.id}
										q={q}
										value={answers[q.id]}
										assumed={assumedBy.get(q.id)}
										guidance={guidanceFor(q.id)}
										onChange={(v) => change(q.id, v)}
									/>
								))}
						</section>
					);
				})}

				<div className="actions">
					{missing.length > 0 && (
						<span className="actions-hint">Still needed: {missing.join(", ")}.</span>
					)}
					<button
						type="button"
						className="btn primary"
						disabled={busy || missing.length > 0}
						onClick={finalise}
					>
						{interim ? "Finalise interim plan" : "Finalise"}
					</button>
				</div>
			</main>
		</div>
	);
}

function QuestionCard({
	q,
	value,
	assumed,
	guidance,
	onChange,
}: {
	q: QuestionView;
	value: AnswerValue | undefined;
	/// The option the engine filled in because this was left blank. Shown as
	/// chosen but muted, so it reads as the tool's guess rather than an answer.
	assumed: string | undefined;
	guidance: string | undefined;
	onChange: (v: AnswerValue) => void;
}) {
	const effective = isAnswered(value) ? value : assumed;
	const selectedIds = new Set(
		Array.isArray(effective)
			? effective
			: typeof effective === "string" && effective
				? [effective]
				: [],
	);
	const warning = q.options.find((o) => selectedIds.has(o.id) && o.warn)?.warn;

	return (
		<div className="card">
			<h3 className="qtitle">{q.label}</h3>
			{q.help && (
				<p className="qhelp">
					<Markup text={q.help} />
				</p>
			)}
			{guidance && (
				<div className="guide">
					<Markup text={guidance} />
				</div>
			)}

			{q.kind === "Band" ? (
				<div className="bandrow">
					{q.options.map((o) => (
						<button
							type="button"
							key={o.id}
							className={`band${selectedIds.has(o.id) ? " on" : ""}`}
							onClick={() => onChange(o.id)}
						>
							{o.label}
						</button>
					))}
				</div>
			) : (
				<div className="choices">
					{q.options.map((o) => {
						const selected = selectedIds.has(o.id);
						return (
							<button
								type="button"
								key={o.id}
								className={`choice${selected ? " on" : ""}`}
								onClick={() => onChange(q.kind === "Multi" ? toggleMulti(q, value, o) : o.id)}
							>
								<span className="choice-tick">{selected && <Check size={13} />}</span>
								<span>
									<span className="choice-title">{o.label}</span>
									{o.note && (
										<span className="choice-note">
											<Markup text={o.note} />
										</span>
									)}
								</span>
							</button>
						);
					})}
				</div>
			)}

			{warning && (
				<div className="warn">
					<Markup text={warning} />
				</div>
			)}
		</div>
	);
}

// Toggle an option in a multi-select, honouring exclusivity: an exclusive
// option ("none of these") clears the rest, and any other clears the exclusive ones.
function toggleMulti(q: QuestionView, value: AnswerValue | undefined, opt: Opt): string[] {
	const current = Array.isArray(value) ? value : [];
	if (current.includes(opt.id)) return current.filter((x) => x !== opt.id);
	if (opt.exclusive) return [opt.id];
	const exclusiveIds = new Set(q.options.filter((o) => o.exclusive).map((o) => o.id));
	return [...current.filter((x) => !exclusiveIds.has(x)), opt.id];
}
