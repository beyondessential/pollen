import { useEffect, useState } from "react";

import { callPricing, money } from "../edition";
import type { AppView, CloneMode, PlanCost, PricingModelView } from "../types";
import { CLONE_OPTIONS, CostSummary, Segmented, Slider } from "./Cost";

/// What the person pricing sets beside a plan (spec COST, The priced wizard).
/// Held by the page and never stored with the plan.
export type PlanSettings = {
	/// Facility servers in BES cloud, or null to use the guess.
	facilities: number | null;
	clone: CloneMode;
};

export const DEFAULT_SETTINGS: PlanSettings = { facilities: null, clone: "standard" };

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
				overrides: { cloud_facilities: facilities, clone: settings.clone, clone_uptime: null },
			},
			controller.signal,
		)
			.then(setPlan)
			.catch(() => {});
		return () => controller.abort();
	}, [pricingPath, view.id, view.evaluation, facilities, settings.clone]);

	return plan;
}

/// The cost beside a plan: the figure, what it rests on, and the clone setting.
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
			<CostSummary cost={plan.cost} missing={plan.missing} compact />

			{plan.open.map((o) => (
				<p key={o.id} className="cost-note">
					{o.label} is not settled, so it is {o.included ? "included" : "left out"}. It is{" "}
					{money(o.monthly_usd)} a month.
				</p>
			))}
			{plan.notes.map((n) => (
				<p key={n} className="cost-note">
					{n}
				</p>
			))}

			<div className="cost-control">
				<span className="cost-control-k">Clone</span>
				<Segmented
					label="Clone"
					options={CLONE_OPTIONS}
					value={settings.clone}
					onChange={(clone) => onSettings({ ...settings, clone })}
				/>
			</div>
		</div>
	);
}

/// How many facility servers run in BES cloud, starting at the guess. Takes
/// the place of the balance question in the flow (spec COST, The priced wizard).
export function FacilityServersControl({
	plan,
	model,
	settings,
	onSettings,
}: {
	plan: PlanCost | null;
	model: PricingModelView;
	settings: PlanSettings;
	onSettings: (s: PlanSettings) => void;
}) {
	const guess = plan?.guess.count ?? 0;
	const count = settings.facilities ?? guess;
	return (
		<div className="cost-control cost-control-flow">
			<span className="cost-control-k">
				{settings.facilities == null ? (
					<span className="cost-control-hint" title={plan?.guess.basis}>
						Guessed from the facilities answer
					</span>
				) : (
					<button
						type="button"
						className="cost-control-reset"
						onClick={() => onSettings({ ...settings, facilities: null })}
					>
						Use guess ({guess})
					</button>
				)}
			</span>
			<Slider
				label="Facility servers in BES cloud"
				value={count}
				min={0}
				max={Math.max(model.max_cloud_facilities, count)}
				onChange={(facilities) => onSettings({ ...settings, facilities })}
			/>
		</div>
	);
}

/// The slider as a question card, for the wizard's flow.
export function FacilityServersCard(props: Parameters<typeof FacilityServersControl>[0]) {
	return (
		<div className="card">
			<h3 className="qtitle">How many facility servers will BES host?</h3>
			<FacilityServersControl {...props} />
		</div>
	);
}
