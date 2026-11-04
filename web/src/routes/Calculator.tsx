import { type ReactNode, useEffect, useState } from "react";

import { CostSummary, Segmented, Slider } from "../components/Cost";
import { Chevron } from "../components/visuals";
import { callPricing, useEdition } from "../edition";
import type { CostBreakdown, CostInput, ParamView, PricingModelView, Unit } from "../types";

type FacilityMode = "cloud" | "client" | "mix";

/// The estimator's answers: option ids by sizing question, and whether other
/// systems connect.
type Estimate = { answers: Record<string, string>; integrations: boolean };
const NO_ESTIMATE: Estimate = { answers: {}, integrations: false };

/// Price a handful of inputs directly (spec COST, The calculator).
export default function Calculator({ model }: { model: PricingModelView }) {
	const { pricingPath } = useEdition();
	const [band, setBand] = useState(model.bands[0]?.name ?? "");
	const [estimate, setEstimate] = useState<Estimate>(NO_ESTIMATE);
	// Facility servers if they all ran in BES cloud, from the estimator's
	// facilities answer. Without one, the band's typical number stands in.
	const [estimated, setEstimated] = useState<number | null>(null);
	const [centralInCloud, setCentralInCloud] = useState(true);
	const [facilityMode, setFacilityMode] = useState<FacilityMode>("client");
	const [mixCount, setMixCount] = useState<number | null>(null);
	const [backups, setBackups] = useState(true);
	const [smokeTesting, setSmokeTesting] = useState(true);
	const [tupaia, setTupaia] = useState(true);
	const [estimating, setEstimating] = useState(false);

	const [cost, setCost] = useState<CostBreakdown | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [showAssumptions, setShowAssumptions] = useState(false);

	const bandIx = Math.max(0, model.bands.findIndex((b) => b.name === band));
	const allInCloud = estimated ?? model.facility_guess_by_band[bandIx] ?? 0;
	const mixDefault = Math.round(allInCloud * model.mix_share);
	const cloudFacilities =
		facilityMode === "cloud" ? allInCloud : facilityMode === "mix" ? (mixCount ?? mixDefault) : 0;

	const input: CostInput = {
		band,
		central_in_cloud: centralInCloud,
		cloud_facilities: cloudFacilities,
		clone: smokeTesting ? "standard" : "off",
		clone_uptime: null,
		backups,
		tupaia,
		windows_central: false,
		windows_facilities: 0,
		windows_clone: false,
	};

	useEffect(() => {
		if (!pricingPath) return;
		const controller = new AbortController();
		callPricing(pricingPath, "cost", input, controller.signal)
			.then((c) => {
				setCost(c);
				setError(null);
			})
			.catch((e: unknown) => {
				if (!controller.signal.aborted) setError(e instanceof Error ? e.message : String(e));
			});
		return () => controller.abort();
		// biome-ignore lint/correctness/useExhaustiveDependencies: input is built from the state listed
	}, [pricingPath, band, centralInCloud, cloudFacilities, smokeTesting, backups, tupaia]);

	// The band comes from the server, which applies the wizard's own derivation.
	function estimateSize(next: Estimate) {
		setEstimate(next);
		if (!pricingPath) return;
		callPricing(pricingPath, "size", next)
			.then((r) => {
				setEstimated(r.cloud_facilities);
				if (r.band) setBand(r.band);
			})
			.catch(() => {});
	}

	// Choosing a band directly replaces whatever was estimated.
	function chooseBand(name: string) {
		setBand(name);
		setEstimate(NO_ESTIMATE);
		setEstimated(null);
	}

	const yesNo = [
		{ id: "yes" as const, label: "Yes" },
		{ id: "no" as const, label: "No" },
	];
	const yn = (b: boolean) => (b ? "yes" : "no");

	return (
		<div className="calc">
			<main className="calc-main">
				<h1 className="calc-title">Hosting cost calculator</h1>

				<div className="card">
					<h3 className="qtitle">Deployment</h3>
					<Field label="Size">
						<div className="calc-stack">
							<Segmented
								label="Size"
								options={model.bands.map((b) => ({ id: b.name, label: b.name }))}
								value={band}
								onChange={chooseBand}
							/>
							<button
								type="button"
								className={`calc-disclose${estimating ? " on" : ""}`}
								aria-expanded={estimating}
								onClick={() => setEstimating(!estimating)}
							>
								<Chevron size={14} />
								<span>Estimate size</span>
							</button>
						</div>
					</Field>
					{estimating && (
						<Estimator
							model={model}
							estimate={estimate}
							onChange={estimateSize}
						/>
					)}
					<Field label="Central server">
						<Segmented
							label="Central server"
							options={[
								{ id: "cloud", label: "BES cloud" },
								{ id: "client", label: "Client hosted" },
							]}
							value={centralInCloud ? "cloud" : "client"}
							onChange={(v) => setCentralInCloud(v === "cloud")}
						/>
					</Field>
					<Field label="Facility servers">
						<div className="calc-stack">
							<Segmented
								label="Facility servers"
								options={[
									{ id: "cloud" as const, label: "BES cloud" },
									{ id: "client" as const, label: "Client hosted" },
									{ id: "mix" as const, label: "A mix" },
								]}
								value={facilityMode}
								onChange={setFacilityMode}
							/>
							{facilityMode === "mix" && (
								<Slider
									label="Facility servers in BES cloud"
									value={cloudFacilities}
									min={0}
									max={Math.max(model.max_cloud_facilities, cloudFacilities)}
									onChange={setMixCount}
								/>
							)}
						</div>
					</Field>
				</div>

				<div className="card">
					<h3 className="qtitle">Services</h3>
					<Field label="BES takes backups">
						<Segmented
							label="BES takes backups"
							options={yesNo}
							value={yn(backups)}
							onChange={(v) => setBackups(v === "yes")}
						/>
					</Field>
					<Field label="Upgrade smoke testing on a clone">
						<Segmented
							label="Upgrade smoke testing on a clone"
							options={yesNo}
							value={yn(smokeTesting)}
							onChange={(v) => setSmokeTesting(v === "yes")}
						/>
					</Field>
					<Field label="Tupaia reporting">
						<Segmented
							label="Tupaia reporting"
							options={yesNo}
							value={yn(tupaia)}
							onChange={(v) => setTupaia(v === "yes")}
						/>
					</Field>
				</div>
			</main>

			<aside className="calc-result">
				<div className="card">
					<CostSummary cost={cost} missing={error} />
				</div>

				<section className="assumptions">
					<button
						type="button"
						className={`qexpand${showAssumptions ? " on" : ""}`}
						aria-expanded={showAssumptions}
						onClick={() => setShowAssumptions(!showAssumptions)}
					>
						<Chevron size={17} />
						<span>Assumptions</span>
					</button>
					{showAssumptions && <Assumptions model={model} />}
				</section>
			</aside>
		</div>
	);
}

/// The sizing questions, each a row of choices like the wizard's. Choosing the
/// selected option again clears it, which is how a question goes back to "not
/// sure".
function Estimator({
	model,
	estimate,
	onChange,
}: {
	model: PricingModelView;
	estimate: Estimate;
	onChange: (e: Estimate) => void;
}) {
	const { sizing, integrations } = model.estimator;
	return (
		<div className="estimator">
			{sizing.map((q) => {
				const chosen = estimate.answers[q.id];
				return (
					<div key={q.id} className="est-q" role="group" aria-label={q.label}>
						<span className="est-label">{q.label}</span>
						<div className="bandrow est-row">
							{q.options.map((o) => (
									<button
										type="button"
										key={o.id}
										aria-pressed={chosen === o.id}
										className={`band${chosen === o.id ? " on" : ""}`}
										onClick={() => {
											const answers = { ...estimate.answers };
											if (chosen === o.id) delete answers[q.id];
											else answers[q.id] = o.id;
											onChange({ ...estimate, answers });
										}}
									>
										{o.label}
									</button>
								))}
						</div>
					</div>
				);
			})}
			{integrations && (
				<div className="est-q" role="group" aria-label={integrations.label}>
					<span className="est-label">{integrations.label}</span>
					<span className="est-hint">{integrations.hint}</span>
					<div className="bandrow est-row est-row-two">
						{[
							{ id: "no", label: "No" },
							{ id: "yes", label: "Yes" },
						].map((o) => {
							const on = estimate.integrations === (o.id === "yes");
							return (
								<button
									type="button"
									key={o.id}
									aria-pressed={on}
									className={`band${on ? " on" : ""}`}
									onClick={() => onChange({ ...estimate, integrations: o.id === "yes" })}
								>
									{o.label}
								</button>
							);
						})}
					</div>
				</div>
			)}
		</div>
	);
}

function Field({ label, children }: { label: string; children: ReactNode }) {
	return (
		<div className="calc-field">
			<span className="calc-label">{label}</span>
			{children}
		</div>
	);
}

function Assumptions({ model }: { model: PricingModelView }) {
	return (
		<div className="card assumptions-body">
			<table className="assume">
				<thead>
					<tr>
						<th>Size</th>
						<th>Server</th>
						<th>Storage</th>
					</tr>
				</thead>
				<tbody>
					{[...model.bands, model.facility].map((b) => (
						<tr key={b.name}>
							<td>{b.name}</td>
							<td>{b.instance}</td>
							<td>{b.volume_gb} GB</td>
						</tr>
					))}
				</tbody>
			</table>
			<ul className="params">
				{model.params.map((p) => (
					<Param key={p.id} p={p} />
				))}
			</ul>
		</div>
	);
}

function Param({ p }: { p: ParamView }) {
	return (
		<li className="param">
			<div className="param-head">
				<span className="param-label">{p.label}</span>
				<span className="param-v">{formatParam(p.value, p.unit)}</span>
			</div>
			<span className={`conf conf-${p.confidence.toLowerCase()}`}>{p.confidence}</span>
			<span className="param-basis">{p.basis}</span>
		</li>
	);
}

function formatParam(value: number, unit: Unit): string {
	switch (unit) {
		case "Usd":
			return `$${value.toLocaleString("en-AU")}`;
		case "UsdPerGbMonth":
			return `$${value} per GB-month`;
		case "Share":
			return `${Math.round(value * 100)}%`;
		case "Multiple":
			return `${value}×`;
		case "Count":
			return String(value);
	}
}
