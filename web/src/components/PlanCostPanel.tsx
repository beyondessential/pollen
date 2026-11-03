import { useEffect, useState } from "react";

import { callPricing, money } from "../edition";
import type { AppView, CloneMode, PlanCost, PricingModelView } from "../types";
import { CLONE_OPTIONS, CostSummary, Segmented, Slider } from "./Cost";

/// The running cost beside a plan in the priced edition (spec COST, The priced
/// wizard). The facility server count and clone mode are the pricer's own
/// overrides, held here and never stored with the plan.
export default function PlanCostPanel({
	pricingPath,
	model,
	view,
}: {
	pricingPath: string;
	model: PricingModelView;
	view: AppView;
}) {
	const [facilities, setFacilities] = useState<number | null>(null);
	const [clone, setClone] = useState<CloneMode>("standard");
	const [plan, setPlan] = useState<PlanCost | null>(null);

	// Keyed on the evaluation, which changes only once the server has stored
	// and re-evaluated the answers, so the cost never reads a stale draft.
	useEffect(() => {
		const controller = new AbortController();
		callPricing(
			pricingPath,
			"plan",
			{ id: view.id, overrides: { cloud_facilities: facilities, clone, clone_uptime: null } },
			controller.signal,
		)
			.then(setPlan)
			.catch(() => {});
		return () => controller.abort();
	}, [pricingPath, view.id, view.evaluation, facilities, clone]);

	if (!plan) return null;
	const count = facilities ?? plan.guess.count;

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
				<span className="cost-control-k">
					Cloud facility servers
					{facilities == null ? (
						<span className="cost-control-hint" title={plan.guess.basis}>
							guessed
						</span>
					) : (
						<button
							type="button"
							className="cost-control-reset"
							onClick={() => setFacilities(null)}
						>
							Use guess ({plan.guess.count})
						</button>
					)}
				</span>
				<Slider
					label="Cloud facility servers"
					value={count}
					min={0}
					max={Math.max(model.max_cloud_facilities, count)}
					onChange={setFacilities}
				/>
			</div>
			<div className="cost-control">
				<span className="cost-control-k">Clone</span>
				<Segmented label="Clone" options={CLONE_OPTIONS} value={clone} onChange={setClone} />
			</div>
		</div>
	);
}
