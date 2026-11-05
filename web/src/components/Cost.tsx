import type { CSSProperties } from "react";

import { money } from "../edition";
import type { CostBreakdown } from "../types";

/// The headline figure, its annual equivalent, and the lines it is made of
/// (spec COST, What a figure is). The shown lines add up to the headline.
export function CostSummary({
	cost,
	missing,
	compact = false,
}: {
	cost: CostBreakdown | null;
	/// Why there is no figure yet, shown in its place.
	missing?: string | null;
	compact?: boolean;
}) {
	const d = cost?.display;
	const marginNote = d && d.margin > 0 ? ` incl. ${Math.round(d.margin * 100)}% margin` : "";
	return (
		<div className={`cost${compact ? " cost-compact" : ""}`}>
			<span className="stat-k">Monthly cost to BES{marginNote}</span>
			{cost && d ? (
				<>
					<span className="cost-v">{money(d.monthly, d.currency)}</span>
					<span className="cost-annual">{money(d.annual, d.currency)} a year</span>
					<ul className="cost-lines">
						{cost.lines.map((l, i) =>
							d.lines[i] > 0 ? (
								<li key={l.id}>
									<span className="cost-line-label">{l.label}</span>
									<span className="cost-line-v">{money(d.lines[i], d.currency)}</span>
									<LineDetail detail={l.detail} confidence={l.confidence} />
								</li>
							) : null,
						)}
					</ul>
				</>
			) : (
				// One quiet message, not a second heading.
				<span className="cost-annual">{missing ?? "Not priced"}</span>
			)}
		</div>
	);
}

/// What a line is made of, marked where it rests on an estimate or an
/// assumption so that is not mistaken for a fact.
function LineDetail({
	detail,
	confidence,
}: {
	detail: string | null;
	confidence: CostBreakdown["lines"][number]["confidence"];
}) {
	const parts = [
		detail,
		confidence === "Estimated" ? "Estimate" : confidence === "Assumed" ? "Assumed" : null,
	].filter((p): p is string => !!p);
	// The dot stays with the qualifier when the detail wraps.
	return parts.length > 0 ? (
		<span className="cost-line-detail">{parts.join(" ·\u00a0")}</span>
	) : null;
}

/// The one line that says what a figure is, and which model it comes from.
/// With a margin applied it is no longer at cost, so it stops saying it is not
/// a price.
export function CostCaveat({ cost }: { cost: CostBreakdown | null | undefined }) {
	if (!cost) return null;
	const { display } = cost;
	return (
		<p className="cost-caveat">
			{display.margin > 0
				? `${display.currency}, including a ${Math.round(display.margin * 100)}% margin.`
				: `${display.currency} at cost. Not a price to the client.`}{" "}
			Model {cost.model_version}.
		</p>
	);
}

/// A row of mutually exclusive options, styled as the wizard's size bands.
/// `compact` sizes the buttons to their labels for an on/off choice.
export function Segmented<T extends string>({
	options,
	value,
	onChange,
	disabled = false,
	label,
	compact = false,
}: {
	options: { id: T; label: string }[];
	value: T;
	onChange: (v: T) => void;
	disabled?: boolean;
	label: string;
	compact?: boolean;
}) {
	return (
		<div
			className={`bandrow${compact ? " bandrow-compact" : ""}${options.length === 3 ? " bandrow-three" : ""}`}
			role="radiogroup"
			aria-label={label}
		>
			{options.map((o) => (
				<button
					type="button"
					key={o.id}
					role="radio"
					aria-checked={value === o.id}
					disabled={disabled}
					className={`band${value === o.id ? " on" : ""}`}
					onClick={() => onChange(o.id)}
				>
					{o.label}
				</button>
			))}
		</div>
	);
}

export const YES_NO = [
	{ id: "yes" as const, label: "Yes" },
	{ id: "no" as const, label: "No" },
];

/// An on/off choice as Yes and No.
export function YesNo({
	value,
	onChange,
	label,
}: {
	value: boolean;
	onChange: (v: boolean) => void;
	label: string;
}) {
	return (
		<Segmented
			compact
			label={label}
			options={YES_NO}
			value={value ? "yes" : "no"}
			onChange={(v) => onChange(v === "yes")}
		/>
	);
}

/// A slider with its reading above the track and what each end means below it.
export function Slider({
	value,
	min,
	max,
	step = 1,
	onChange,
	format = String,
	label,
	disabled = false,
	startLabel,
	endLabel,
}: {
	value: number;
	min: number;
	max: number;
	step?: number;
	onChange: (v: number) => void;
	format?: (v: number) => string;
	label: string;
	disabled?: boolean;
	startLabel?: string;
	endLabel?: string;
}) {
	// How far along the track the thumb is, from 0 to 1, for the fill.
	const at = max > min ? (value - min) / (max - min) : 0;
	return (
		<div className="slider">
			<span className="slider-v">{format(value)}</span>
			<input
				style={{ "--at": at } as CSSProperties}
				type="range"
				aria-label={label}
				min={min}
				max={max}
				step={step}
				value={value}
				disabled={disabled}
				onChange={(e) => onChange(Number(e.target.value))}
			/>
			{(startLabel || endLabel) && (
				<div className="slider-ends">
					<span>{startLabel}</span>
					<span>{endLabel}</span>
				</div>
			)}
		</div>
	);
}
