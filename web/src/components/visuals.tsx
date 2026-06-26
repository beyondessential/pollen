import type { ReactNode } from "react";

import {
	CONSEQUENCE_TYPE_LABEL,
	type Consequence,
	type ConsequenceType,
	STATUS_LABEL,
	type Severity,
	type Verdict,
} from "../types";

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
				<p className="cons-detail">{c.detail}</p>
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
function verdictMeta(verdict: Verdict): VerdictMeta {
	switch (verdict) {
		case "Blocking":
			return {
				color: "var(--block)",
				bg: "var(--block-bg)",
				title: "Very likely not possible as specified",
			};
		case "NonDefault":
			return {
				color: "var(--offdef)",
				bg: "var(--offdef-bg)",
				title: "Possible, with acknowledged off-default choices",
			};
		default:
			return {
				color: "var(--clear)",
				bg: "var(--clear-bg)",
				title: "On the default, supported path",
			};
	}
}

export function VerdictBanner({
	verdict,
	offDefault,
	blocking,
	started = true,
	big = false,
}: {
	verdict: Verdict;
	offDefault: number;
	blocking: number;
	/// Whether any choice has been made yet. Before then there's no verdict to
	/// report — only an empty form.
	started?: boolean;
	big?: boolean;
}) {
	if (!started) {
		return (
			<div
				className={`verdict${big ? " verdict-big" : ""}`}
				style={{ background: "var(--line-soft)", color: "var(--ink-soft)" }}
			>
				<div>
					<div className="verdict-t">Nothing recorded yet</div>
					<div className="verdict-s">Make a choice and its consequences appear here.</div>
				</div>
			</div>
		);
	}
	const m = verdictMeta(verdict);
	const subtitle =
		verdict === "Blocking"
			? `${blocking} blocking conflict${blocking === 1 ? "" : "s"} — something must change. Everything is still recorded below.`
			: verdict === "NonDefault"
				? `${offDefault} choice${offDefault === 1 ? "" : "s"} off the default path. This will be harder to support.`
				: "No off-default choices recorded.";
	return (
		<div
			className={`verdict${big ? " verdict-big" : ""}`}
			style={{ background: m.bg, color: m.color }}
		>
			<div>
				<div className="verdict-t">{m.title}</div>
				<div className="verdict-s">{subtitle}</div>
			</div>
		</div>
	);
}
