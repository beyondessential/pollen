import { useMemo, useState } from "react";

import { callApi } from "../api";
import { Markup } from "../markup";
import {
	type AnswerValue,
	type AppView,
	asShares,
	isAnswered,
	type Opt,
	type QuestionView,
	type Section,
} from "../types";
import { Check, ConsequenceCard, VerdictBanner } from "./visuals";

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
	const started = Object.values(answers).some(isAnswered);
	const byId = new Map(view.questions.map((q) => [q.id, q]));
	const assumedBy = new Map(ev.assumed.map((a) => [a.question, a.option]));
	const openItems = new Set(ev.open_items);
	const offDefault = ev.consequences.filter(
		(c) => c.consequence.severity === "NonDefault",
	).length;
	const blocking = ev.consequences.filter((c) => c.consequence.severity === "Blocking").length;
	const guidanceFor = (qid: string) => ev.guidance.find((g) => g.at === qid)?.message;

	// Most-severe first in the ledger; stable within a severity (ruleset order).
	const severityRank = { Blocking: 0, NonDefault: 1, Default: 2 };
	const ledger = [...ev.consequences].sort(
		(a, b) => severityRank[a.consequence.severity] - severityRank[b.consequence.severity],
	);

	// Group the visible questions into their sections, keeping ruleset order
	// within each. A question naming no section falls into the first one.
	const grouped = useMemo(() => {
		const sections: Section[] = view.sections.length
			? view.sections
			: [{ id: "", label: "", blurb: null, collapsed: false }];
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
				<VerdictBanner
					verdict={ev.verdict}
					offDefault={offDefault}
					blocking={blocking}
					openItems={ev.open_items.length}
					started={started}
				/>
				<div className="meters">
					<div className="meter">
						<span className="meter-k">Size</span>
						<span className="meter-v">{ev.derived["size"] ?? "—"}</span>
					</div>
					<div className="meter">
						<span className="meter-k">Custom</span>
						<span className="meter-v" style={{ color: offDefault ? "var(--offdef)" : undefined }}>
							{offDefault}
						</span>
					</div>
					<div className="meter">
						<span className="meter-k">Open</span>
						<span className="meter-v" style={{ color: interim ? "var(--open)" : undefined }}>
							{ev.open_items.length}
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
					{ledger.length === 0 ? (
						<p className="ledger-empty">Nothing yet.</p>
					) : (
						ledger.map((c) => <ConsequenceCard key={c.id} c={c.consequence} />)
					)}
				</div>
			</aside>

			<main className="main">
				{grouped.map(({ section, questions }) => {
					const isOpen = opened[section.id] ?? !section.collapsed;
					const assumedHere = questions.filter((q) => assumedBy.has(q.id)).length;
					const openHere = questions.filter((q) => openItems.has(q.id)).length;
					return (
						<section className="qsection" key={section.id || "all"}>
							{section.label && (
								<div className="qsection-head">
									<div>
										<h2 className="qsection-title">{section.label}</h2>
										{section.blurb && <p className="qsection-blurb">{section.blurb}</p>}
									</div>
									{section.collapsed && (
										<button
											type="button"
											className="btn ghost"
											onClick={() => setOpened({ ...opened, [section.id]: !isOpen })}
										>
											{isOpen ? "Hide" : "Show"}
										</button>
									)}
								</div>
							)}
							{!isOpen ? (
								<p className="qsection-summary">
									{assumedHere > 0 && `${assumedHere} taking the standard setup`}
									{assumedHere > 0 && openHere > 0 && " · "}
									{openHere > 0 && `${openHere} left for BES to confirm`}
								</p>
							) : (
								questions.map((q) => (
									<QuestionCard
										key={q.id}
										q={q}
										value={answers[q.id]}
										assumed={assumedBy.get(q.id)}
										open={openItems.has(q.id)}
										guidance={guidanceFor(q.id)}
										onChange={(v) => change(q.id, v)}
									/>
								))
							)}
						</section>
					);
				})}

				<div className="actions">
					{missing.length > 0 ? (
						<span className="actions-hint">Still needed: {missing.join(", ")}.</span>
					) : interim ? (
						<span className="actions-hint">
							{ev.open_items.length} question{ev.open_items.length === 1 ? "" : "s"} left open —
							this finalises as an interim plan you can complete later.
						</span>
					) : null}
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
	open,
	guidance,
	onChange,
}: {
	q: QuestionView;
	value: AnswerValue | undefined;
	/// The option the engine filled in because this was left blank.
	assumed: string | undefined;
	/// Whether this is recorded as an open item rather than answered.
	open: boolean;
	guidance: string | undefined;
	onChange: (v: AnswerValue) => void;
}) {
	// What's effectively chosen: the user's answer, or the assumed default.
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
		<div className={`card${open ? " card-open" : ""}`}>
			<div className="qhead">
				<h3 className="qtitle">{q.label}</h3>
				{assumed && <span className="qflag qflag-assumed">Assumed</span>}
				{open && <span className="qflag qflag-open">For BES to confirm</span>}
			</div>
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

			{q.kind === "Mix" ? (
				<MixControl q={q} value={value} assumed={assumed} onChange={onChange} />
			) : q.kind === "Band" ? (
				<div className="bandrow">
					{q.options.map((o) => (
						<button
							type="button"
							key={o.id}
							className={`band${selectedIds.has(o.id) ? (assumed ? " on faint" : " on") : ""}${
								o.unsure ? " band-unsure" : ""
							}`}
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
								className={`choice${selected ? (assumed ? " on faint" : " on") : ""}${
									o.unsure ? " choice-unsure" : ""
								}`}
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
			{assumed && (
				<p className="qassumed">
					Left blank, so the standard setup is assumed. Change it if that's wrong.
				</p>
			)}
		</div>
	);
}

/// A rough percentage split across the options. Moving one share redistributes
/// the remainder across the others in proportion, so the total always reads 100.
function MixControl({
	q,
	value,
	assumed,
	onChange,
}: {
	q: QuestionView;
	value: AnswerValue | undefined;
	assumed: string | undefined;
	onChange: (v: AnswerValue) => void;
}) {
	const shares = isAnswered(value)
		? asShares(value)
		: Object.fromEntries(q.options.map((o) => [o.id, o.id === assumed ? 100 : 0]));

	return (
		<div className="mix">
			{q.options.map((o) => {
				const share = shares[o.id] ?? 0;
				return (
					<div className={`mix-row${share > 0 ? " on" : ""}`} key={o.id}>
						<div className="mix-head">
							<span className="mix-label">{o.label}</span>
							<span className="mix-pct">{share}%</span>
						</div>
						<input
							className="mix-range"
							type="range"
							min={0}
							max={100}
							step={5}
							value={share}
							aria-label={o.label}
							onChange={(e) => onChange(redistribute(q.options, shares, o.id, +e.target.value))}
						/>
						{o.note && (
							<p className="mix-note">
								<Markup text={o.note} />
							</p>
						)}
					</div>
				);
			})}
		</div>
	);
}

/// Set one option's share and spread the remaining percentage over the others,
/// in proportion to what they already hold (evenly, when they hold nothing).
/// The last option absorbs the rounding so the shares total exactly 100.
function redistribute(
	options: Opt[],
	shares: Record<string, number>,
	id: string,
	raw: number,
): Record<string, number> {
	const value = Math.max(0, Math.min(100, Math.round(raw)));
	const others = options.filter((o) => o.id !== id);
	const next: Record<string, number> = { [id]: value };
	if (others.length === 0) return { [id]: 100 };

	const remaining = 100 - value;
	const total = others.reduce((sum, o) => sum + (shares[o.id] ?? 0), 0);
	let allocated = 0;
	others.forEach((o, i) => {
		const last = i === others.length - 1;
		const portion = last
			? remaining - allocated
			: total === 0
				? Math.round(remaining / others.length)
				: Math.round(((shares[o.id] ?? 0) / total) * remaining);
		next[o.id] = Math.max(0, portion);
		allocated += next[o.id];
	});
	return next;
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
