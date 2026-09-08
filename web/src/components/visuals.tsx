import { useState } from "react";

import type { Consequence } from "../types";
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

/// The viability callout. It exists to flag a configuration that will not work,
/// so it speaks only when there is a blocking conflict. Choices that are merely
/// off the standard path are listed in full in their own group, and a banner
/// counting them again would be saying it twice.
export function VerdictBanner({ conflicts }: { conflicts: string[] }) {
	const [open, setOpen] = useState(true);
	if (conflicts.length === 0) return null;
	return (
		<div
			className="verdict verdict-big"
			style={{ background: "var(--block-bg)", color: "var(--block)" }}
		>
			<button
				type="button"
				className={`verdict-toggle${open ? " on" : ""}`}
				aria-expanded={open}
				onClick={() => setOpen(!open)}
			>
				<Chevron size={16} />
				<span className="verdict-t">
					{conflicts.length} blocking conflict{conflicts.length === 1 ? "" : "s"}: this will not
					work as specified
				</span>
			</button>
			{/* Always rendered, hidden with CSS, so it prints whole. */}
			<ul className={`verdict-list${open ? "" : " shut"}`}>
				{conflicts.map((title) => (
					<li key={title}>{title}</li>
				))}
			</ul>
		</div>
	);
}
