import { useEffect, useState } from "react";

import { callPricing, money } from "../edition";
import type { AppView, CloneMode, PlanCost } from "../types";
import { CostCaveat, CostSummary, Slider, YesNo } from "./Cost";

/// What the person pricing sets beside a plan (spec COST, The priced wizard).
/// Held by the page and never stored with the plan.
export type PlanSettings = {
	/// Facility servers in BES cloud, or null to use the guess.
	facilities: number | null;
	clone: CloneMode;
	/// A demo/training environment for the deployment, on unless declined.
	demoTraining: boolean;
};

export const DEFAULT_SETTINGS: PlanSettings = {
	facilities: null,
	clone: "standard",
	demoTraining: true,
};

/// Price a plan as its answers and the pricer's settings stand. `facilities`
/// is sent only while the slider is showing, so a stale setting cannot move a
/// figure once the slider has gone.
export function usePlanCost(
	pricingPath: string | null,
	view: AppView,
	settings: PlanSettings,
	sliderShowing: boolean,
): PlanCost | null {
	const [plan, setPlan] = useState<PlanCost | null>(null);
	const facilities = sliderShowing ? settings.facilities : null;

	// Keyed on the evaluation, which changes only once the server has stored
	// and re-evaluated the answers, so the cost never reads a stale draft.
	useEffect(() => {
		if (pricingPath == null) return;
		const controller = new AbortController();
		callPricing(
			pricingPath,
			"plan",
			{
				id: view.id,
				overrides: {
					cloud_facilities: facilities,
					clone: settings.clone,
					clone_uptime: null,
					demo_training: settings.demoTraining,
				},
			},
			controller.signal,
		)
			.then(setPlan)
			// A figure that could not be refreshed is not left standing.
			.catch(() => {
				if (!controller.signal.aborted) setPlan(null);
			});
		return () => controller.abort();
	}, [pricingPath, view.id, view.evaluation, facilities, settings.clone, settings.demoTraining]);

	return plan;
}

/// The figure, what it rests on, and anything still unsettled that bears on it.
export function PlanCostSummary({ plan }: { plan: PlanCost | null }) {
	if (!plan) return null;
	const currency = plan.cost?.display.currency;
	return (
		<>
			<CostSummary cost={plan.cost} missing={plan.missing} compact />
			{plan.open.map((o) => (
				<p key={o.id} className="cost-callout">
					{o.included
						? `${o.label} (${money(o.display_monthly, currency)} a month) is counted while undecided.`
						: `${o.label} would add ${money(o.display_monthly, currency)} a month. Not yet decided.`}
				</p>
			))}
			{plan.notes.map((n) => (
				<p key={n} className="cost-callout">
					{n}
				</p>
			))}
		</>
	);
}

/// The clone setting, since it is not a client choice.
export function PlanCostControls({
	settings,
	onSettings,
}: {
	settings: PlanSettings;
	onSettings: (s: PlanSettings) => void;
}) {
	return (
		<div className="cost-control cost-adjust">
			<span className="stat-k">Adjust</span>
			<span className="cost-control-k">Upgrade smoke testing on a clone</span>
			<YesNo
				label="Upgrade smoke testing on a clone"
				value={settings.clone !== "off"}
				onChange={(on) => onSettings({ ...settings, clone: on ? "standard" : "off" })}
			/>
			<span className="cost-control-k cost-control-gap">Demo/training environment</span>
			<YesNo
				label="Demo/training environment"
				value={settings.demoTraining}
				onChange={(demoTraining) => onSettings({ ...settings, demoTraining })}
			/>
		</div>
	);
}

/// The cost beside a plan, in the wizard's rail: the figure and lines, then
/// the setting that moves them, then what a figure is.
export default function PlanCostPanel({
	plan,
	settings,
	onSettings,
}: {
	plan: PlanCost | null;
	settings: PlanSettings;
	onSettings: (s: PlanSettings) => void;
}) {
	if (!plan) return null;
	return (
		<div className="cost-panel">
			<PlanCostSummary plan={plan} />
			{plan.cost && <PlanCostControls settings={settings} onSettings={onSettings} />}
			<CostCaveat cost={plan.cost} />
		</div>
	);
}

/// How many of the facility servers run in BES cloud, starting at the guess.
/// Takes the place of the balance question in the flow (spec COST, The priced
/// wizard).
export function FacilityServersControl({
	plan,
	settings,
	onSettings,
	label,
}: {
	plan: PlanCost | null;
	settings: PlanSettings;
	onSettings: (s: PlanSettings) => void;
	/// A heading for the control where the question around it is not showing.
	label?: string;
}) {
	// Nothing to show until the plan has been priced.
	if (!plan) return null;
	const guess = plan.guess.count;
	const total = plan.guess.total;
	const count = Math.min(settings.facilities ?? guess, total);
	if (total === 0) {
		return <p className="cost-hint">Answer the facilities question first.</p>;
	}
	return (
		<div className="cost-control cost-control-flow">
			{label && <span className="cost-control-k">{label}</span>}
			<Slider
				label="Facility servers in BES cloud"
				value={count}
				min={0}
				max={total}
				onChange={(facilities) => onSettings({ ...settings, facilities })}
				startLabel="Client hosted"
				endLabel="BES cloud"
				format={(n) => `${n} of ${total} in BES cloud`}
			/>
			<p className="cost-hint">
				{total} facility server{total === 1 ? "" : "s"}, from the facilities answer.
				{settings.facilities != null && (
					<>
						{" "}
						<button
							type="button"
							className="cost-control-reset"
							onClick={() => onSettings({ ...settings, facilities: null })}
						>
							Use {guess}
						</button>
					</>
				)}
			</p>
		</div>
	);
}

/// The slider as a question card, for the wizard's flow.
export function FacilityServersCard(props: Omit<Parameters<typeof FacilityServersControl>[0], "label">) {
	return (
		<div className="card">
			<h3 className="qtitle">How many facility servers run in BES cloud?</h3>
			<FacilityServersControl {...props} />
		</div>
	);
}
