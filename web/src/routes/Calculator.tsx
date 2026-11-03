import { type ReactNode, useEffect, useState } from "react";

import { CLONE_OPTIONS, CostSummary, Segmented, Slider } from "../components/Cost";
import { Chevron } from "../components/visuals";
import { callPricing, useEdition } from "../edition";
import type { CostBreakdown, CostInput, ParamView, PricingModelView, Unit } from "../types";

/// Price a handful of inputs directly (spec COST, The calculator).
export default function Calculator({ model }: { model: PricingModelView }) {
	const { pricingPath } = useEdition();
	const [input, setInput] = useState<CostInput>(() => ({
		band: model.bands[0]?.name ?? "",
		central_in_cloud: true,
		cloud_facilities: 0,
		clone: "standard",
		clone_uptime: model.clone_uptime,
		backups: true,
		tupaia: false,
		windows_central: false,
		windows_facilities: 0,
		windows_clone: false,
	}));
	const [cost, setCost] = useState<CostBreakdown | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [showAssumptions, setShowAssumptions] = useState(false);

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
	}, [pricingPath, input]);

	const set = (patch: Partial<CostInput>) => setInput((prev) => ({ ...prev, ...patch }));
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
						<Segmented
							label="Size"
							options={model.bands.map((b) => ({ id: b.name, label: b.name }))}
							value={input.band}
							onChange={(band) => set({ band })}
						/>
					</Field>
					<Field label="Central server">
						<Segmented
							label="Central server"
							options={[
								{ id: "cloud", label: "BES cloud" },
								{ id: "client", label: "Client hosted" },
							]}
							value={input.central_in_cloud ? "cloud" : "client"}
							onChange={(v) => set({ central_in_cloud: v === "cloud" })}
						/>
					</Field>
					<Field label="Facility servers in BES cloud">
						<Slider
							label="Facility servers in BES cloud"
							value={input.cloud_facilities}
							min={0}
							max={model.max_cloud_facilities}
							onChange={(n) =>
								set({
									cloud_facilities: n,
									windows_facilities: Math.min(input.windows_facilities, n),
								})
							}
						/>
					</Field>
				</div>

				<div className="card">
					<h3 className="qtitle">Services</h3>
					<Field label="Clone">
						<Segmented
							label="Clone"
							options={CLONE_OPTIONS}
							value={input.clone}
							onChange={(clone) => set({ clone })}
						/>
					</Field>
					{input.clone === "standard" && (
						<Field label="Clone running">
							<Slider
								label="Share of the month the clone runs"
								value={Math.round((input.clone_uptime ?? model.clone_uptime) * 100)}
								min={0}
								max={100}
								step={5}
								format={(v) => `${v}%`}
								onChange={(v) => set({ clone_uptime: v / 100 })}
							/>
						</Field>
					)}
					<Field label="BES takes backups">
						<Segmented
							label="BES takes backups"
							options={yesNo}
							value={yn(input.backups)}
							onChange={(v) => set({ backups: v === "yes" })}
						/>
					</Field>
					<Field label="Feeds Tupaia">
						<Segmented
							label="Feeds Tupaia"
							options={yesNo}
							value={yn(input.tupaia)}
							onChange={(v) => set({ tupaia: v === "yes" })}
						/>
					</Field>
				</div>

				<div className="card">
					<h3 className="qtitle">Legacy Windows</h3>
					<p className="qhelp">For existing deployments still on Windows.</p>
					<Field label="Windows central server">
						<Segmented
							label="Windows central server"
							options={yesNo}
							value={yn(input.windows_central && input.central_in_cloud)}
							disabled={!input.central_in_cloud}
							onChange={(v) => set({ windows_central: v === "yes" })}
						/>
					</Field>
					<Field label="Windows facility servers">
						<Slider
							label="Windows facility servers"
							value={input.windows_facilities}
							min={0}
							max={input.cloud_facilities}
							disabled={input.cloud_facilities === 0}
							onChange={(n) => set({ windows_facilities: n })}
						/>
					</Field>
					<Field label="Windows clone">
						<Segmented
							label="Windows clone"
							options={yesNo}
							value={yn(input.windows_clone && input.clone !== "off")}
							disabled={input.clone === "off"}
							onChange={(v) => set({ windows_clone: v === "yes" })}
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
