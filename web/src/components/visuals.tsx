import { useState } from "react";

import type { Consequence, Verdict } from "../types";
import { Markup } from "../markup";

// Minimal inline icons (lucide-style paths), so nothing is fetched at runtime.
function Icon({ paths, size = 16 }: { paths: string; size?: number }) {
	return (
		<svg
			width={size}
			height={size}
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth={2}
			strokeLinecap="round"
			strokeLinejoin="round"
			aria-hidden
			// biome-ignore lint: inline icon path
			dangerouslySetInnerHTML={{ __html: paths }}
		/>
	);
}
export const Check = (p: { size?: number }) => <Icon paths='<path d="M20 6 9 17l-5-5"/>' {...p} />;
export const Chevron = (p: { size?: number }) => (
	<Icon paths='<path d="m6 9 6 6 6-6"/>' {...p} />
);

/// One consequence, with a box the reader ticks off as they deal with it.
export function ConsequenceCard({
	c,
	done,
	onToggle,
}: {
	c: Consequence;
	done: boolean;
	onToggle: () => void;
}) {
	return (
		<div className="item">
			<button
				type="button"
				className={`item-check${done ? " on" : ""}`}
				aria-pressed={done}
				aria-label={done ? `Mark "${c.title}" not done` : `Mark "${c.title}" done`}
				onClick={onToggle}
			>
				{done && <Check size={12} />}
			</button>
			<div className="item-body">
				<h4 className={`item-title${done ? " item-done" : ""}`}>{c.title}</h4>
				<p className="item-detail">
					<Markup text={c.detail} />
				</p>
				{c.cost && (
					<p className="cost-note">
						{c.cost.tier}
						{c.cost.ballpark ? ` · ${c.cost.ballpark}` : ""}
					</p>
				)}
			</div>
		</div>
	);
}

type VerdictMeta = { color: string; bg: string; title: string };

/// The viability callout. It exists to flag a configuration that will not work,
/// so it renders nothing when there is nothing to say. A banner announcing that
/// all is well is noise on every artifact that has no problem.
export function VerdictBanner({
	verdict,
	offDefault,
	blocking,
	choices,
}: {
	verdict: Verdict;
	offDefault: number;
	blocking: number;
	/// What took the plan off the standard path, so a reader can see which
	/// choices rather than only how many.
	choices: string[];
}) {
	const [open, setOpen] = useState(false);
	if (verdict === "Clear") return null;
	const m: VerdictMeta =
		verdict === "Blocking"
			? {
					color: "var(--block)",
					bg: "var(--block-bg)",
					title: `${blocking} blocking conflict${blocking === 1 ? "" : "s"}: this will not work as specified`,
				}
			: {
					color: "var(--offdef)",
					bg: "var(--offdef-bg)",
					title: `${offDefault} choice${offDefault === 1 ? "" : "s"} off the standard path`,
				};
	return (
		<div className="verdict verdict-big" style={{ background: m.bg, color: m.color }}>
			<button
				type="button"
				className={`verdict-toggle${open ? " on" : ""}`}
				aria-expanded={open}
				onClick={() => setOpen(!open)}
			>
				<Chevron size={16} />
				<span className="verdict-t">{m.title}</span>
			</button>
			{/* Always rendered, hidden with CSS, so it prints whole. */}
			<ul className={`verdict-list${open ? "" : " shut"}`}>
				{choices.map((title) => (
					<li key={title}>{title}</li>
				))}
			</ul>
		</div>
	);
}
