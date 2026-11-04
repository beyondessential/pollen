import { money } from "../edition";
import type { CostBreakdown } from "../types";

/// The headline figure, its annual equivalent, and the lines it is made of
/// (spec COST, What a figure is).
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
						{cost.lines.map((l) => (
							<li key={l.id}>
								<span>
									<span className="cost-line-label">{l.label}</span>
									{l.detail && <span className="cost-line-detail">{l.detail}</span>}
								</span>
								<span className="cost-line-v">{money(l.monthly_usd)}</span>
							</li>
						))}
					</ul>
				</>
			) : (
				<>
					<span className="cost-v cost-v-none">Not priced</span>
					{missing && <span className="cost-annual">{missing}</span>}
				</>
			)}
			<p className="cost-caveat">
				Indicative hosting cost to BES, not a price. Lines are USD at cost.
			</p>
		</div>
	);
}

/// A row of mutually exclusive options, styled as the wizard's size bands.
export function Segmented<T extends string>({
	options,
	value,
	onChange,
	disabled = false,
	label,
}: {
	options: { id: T; label: string }[];
	value: T;
	onChange: (v: T) => void;
	disabled?: boolean;
	label: string;
}) {
	return (
		<div className="bandrow" role="radiogroup" aria-label={label}>
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

/// A slider with its value beside it.
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
	/// Names for what each end of the track means.
	startLabel?: string;
	endLabel?: string;
}) {
	return (
		<div className="slider">
			<div className="slider-track">
				<input
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
			<span className="slider-v">{format(value)}</span>
		</div>
	);
}

export const CLONE_OPTIONS = [
	{ id: "off" as const, label: "Off" },
	{ id: "standard" as const, label: "Standard" },
	{ id: "always_on" as const, label: "Always on" },
];
