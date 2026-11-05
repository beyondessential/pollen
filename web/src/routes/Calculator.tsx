import { type ReactNode, useEffect, useRef, useState } from "react";

import { CostCaveat, CostSummary, Segmented, Slider, YesNo } from "../components/Cost";
import { Chevron } from "../components/visuals";
import { callPricing, useEdition } from "../edition";
import type { CostBreakdown, CostInput, PricingModelView } from "../types";

type FacilityMode = "cloud" | "client" | "mix";

/// The estimator's answers: option ids by sizing question.
type Estimate = { answers: Record<string, string> };
const NO_ESTIMATE: Estimate = { answers: {} };

/// Price a handful of inputs directly (spec COST, The calculator).
export default function Calculator({ model }: { model: PricingModelView }) {
	const { pricingPath } = useEdition();
	const [band, setBand] = useState(model.bands[0]?.name ?? "");
	const [estimate, setEstimate] = useState<Estimate>(NO_ESTIMATE);
	// Facility servers the deployment has, from the estimator's facilities
	// answer. Without one, the band's typical number stands in.
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

	const bandIx = Math.max(
		0,
		model.bands.findIndex((b) => b.name === band),
	);
	// All the deployment's facility servers, wherever they run.
	const totalFacilities = estimated ?? model.facility_guess_by_band[bandIx] ?? 0;
	const mixDefault = Math.round(totalFacilities * model.mix_share);
	const cloudFacilities =
		facilityMode === "cloud"
			? totalFacilities
			: facilityMode === "mix"
				? Math.min(mixCount ?? mixDefault, totalFacilities)
				: 0;

	useEffect(() => {
		if (!pricingPath) return;
		const controller = new AbortController();
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
		callPricing(pricingPath, "cost", input, controller.signal)
			.then((c) => {
				setCost(c);
				setError(null);
			})
			.catch((e: unknown) => {
				if (controller.signal.aborted) return;
				// A figure that could not be refreshed is not left standing.
				setCost(null);
				setError(e instanceof Error ? e.message : String(e));
			});
		return () => controller.abort();
	}, [pricingPath, band, centralInCloud, cloudFacilities, smokeTesting, backups, tupaia]);

	// The band comes from the server, which applies the wizard's own derivation.
	// Only the latest reply counts, and choosing a band directly outranks any
	// reply still on its way.
	const latest = useRef(0);
	function estimateSize(next: Estimate) {
		setEstimate(next);
		if (!pricingPath) return;
		const mine = ++latest.current;
		callPricing(pricingPath, "size", next)
			.then((r) => {
				if (mine !== latest.current) return;
				// A band the pricing table lacks is not taken up, nor is the
				// facility count that came with it.
				if (r.band && !model.bands.some((b) => b.name === r.band)) return;
				setEstimated(r.cloud_facilities);
				setMixCount(null);
				if (r.band) setBand(r.band);
			})
			.catch(() => {});
	}

	// Choosing a band directly replaces whatever was estimated.
	function chooseBand(name: string) {
		latest.current++;
		setBand(name);
		setEstimate(NO_ESTIMATE);
		setEstimated(null);
		setMixCount(null);
	}

	const source = estimated != null ? "from the estimate" : `typical for ${band}`;
	const facilityHint =
		facilityMode === "client"
			? null
			: `${totalFacilities} facility server${totalFacilities === 1 ? "" : "s"}, ${source}.`;

	return (
		<div className="calc">
			<h1 className="calc-title">Hosting cost calculator</h1>
			<main className="calc-main">
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
									max={totalFacilities}
									onChange={setMixCount}
									startLabel="Client hosted"
									endLabel="BES cloud"
									format={(n) => `${n} of ${totalFacilities} in BES cloud`}
								/>
							)}
							{facilityHint && <p className="cost-hint">{facilityHint}</p>}
						</div>
					</Field>
				</div>

				<div className="card">
					<h3 className="qtitle">Services</h3>
					<Field label="BES takes backups">
						<YesNo label="BES takes backups" value={backups} onChange={setBackups} />
					</Field>
					<Field label="Upgrade smoke testing on a clone">
						<YesNo
							label="Upgrade smoke testing on a clone"
							value={smokeTesting}
							onChange={setSmokeTesting}
						/>
					</Field>
					<Field label="Tupaia reporting">
						<div className="calc-stack">
							<YesNo label="Tupaia reporting" value={tupaia} onChange={setTupaia} />
							<p className="cost-hint">Over and above backups.</p>
						</div>
					</Field>
				</div>
			</main>

			<aside className="calc-result">
				<div className="card">
					<CostSummary cost={cost} missing={error} />
					<CostCaveat cost={cost} />
				</div>
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
	const { sizing } = model.estimator;
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
