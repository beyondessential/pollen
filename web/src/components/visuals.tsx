import type { ReactNode } from "react";

import {
	CONSEQUENCE_TYPE_LABEL,
	type Consequence,
	type ConsequenceType,
	STATUS_LABEL,
	type Severity,
	type Verdict,
} from "../types";
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

type SevMeta = { color: string; bg: string; dot: string };
const SEVERITY: Record<Severity, SevMeta> = {
	Default: { color: "var(--ink-soft)", bg: "var(--line-soft)", dot: "var(--ink-faint)" },
	NonDefault: { color: "var(--offdef)", bg: "var(--offdef-bg)", dot: "var(--offdef)" },
	Blocking: { color: "var(--block)", bg: "var(--block-bg)", dot: "var(--block)" },
};

const TYPE_COLOR: Record<ConsequenceType, { color: string; bg: string }> = {
	Cost: { color: "#9a6a00", bg: "#fbf1dc" },
	Operational: { color: "#1f5fa6", bg: "#e6eff8" },
	Capability: { color: "#8a2e2e", bg: "#f7e4e1" },
	Support: { color: "#6b3a8a", bg: "#efe6f6" },
};

export function Tag({
	children,
	color,
	bg,
}: {
	children: ReactNode;
	color: string;
	bg: string;
}) {
	return (
		<span className="tag" style={{ color, background: bg }}>
			{children}
		</span>
	);
}

export function ConsequenceCard({ c }: { c: Consequence }) {
	const sev = SEVERITY[c.severity];
	return (
		<div className="cons" style={{ borderColor: sev.bg }}>
			<div className="cons-bar" style={{ background: sev.dot }} />
			<div className="cons-body">
				<div className="cons-head">
					<span className="cons-dot" style={{ background: sev.dot }} />
					<span className="cons-title">{c.title}</span>
				</div>
				<p className="cons-detail">
					<Markup text={c.detail} />
				</p>
				<div className="cons-tags">
					{c.types.map((t) => (
						<Tag key={t} color={TYPE_COLOR[t].color} bg={TYPE_COLOR[t].bg}>
							{CONSEQUENCE_TYPE_LABEL[t]}
						</Tag>
					))}
					<Tag color="#3a4750" bg="#eaedee">
						{STATUS_LABEL[c.status]}
					</Tag>
					{c.cost && (
						<span className="cost-note">
							{c.cost.tier}
							{c.cost.ballpark ? ` · ${c.cost.ballpark}` : ""}
						</span>
					)}
				</div>
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
}: {
	verdict: Verdict;
	offDefault: number;
	blocking: number;
}) {
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
			<div className="verdict-t">{m.title}</div>
		</div>
	);
}
