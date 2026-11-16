import { type ReactNode, useEffect, useState } from "react";

import { Segmented } from "../components/Cost";
import { callMigration, useEdition } from "../edition";
import type {
	MigrationArea,
	MigrationCaseBands,
	MigrationEstimate,
	MigrationEstimateInput,
	MigrationImportEstimate,
	MigrationImportInput,
	MigrationImportProject,
	MigrationModelView,
	MigrationSpan,
} from "../types";

type Route = "bes" | "client";

/// Nothing answered, except the questions with a default, which start on it:
/// no forms, one site, Tamanu not yet in use.
const STARTING_ANSWERS: MigrationEstimateInput = {
	tier: null,
	add: [],
	remove: [],
	form_count: 0,
	registry_count: 0,
	facility_count: 1,
	tamanu_live: false,
	source_kind: null,
	source_instances: null,
	source_db: null,
	doc_dictionary: null,
	doc_codes: null,
	doc_contact: null,
	doc_prior_exports: null,
	patients_per_day: null,
	history_years: null,
	many_users: null,
	review_speed: null,
	staffing: null,
};

const NEW_PROJECT: MigrationImportProject = {
	sites: null,
	registry: null,
	forms: 0,
	cases: null,
};
const NO_PROJECTS: MigrationImportInput = {
	templates_exist: null,
	projects: [NEW_PROJECT],
};

const DOC_LEVELS = [
	{ id: "good" as const, label: "Yes" },
	{ id: "partial" as const, label: "Partly" },
	{ id: "none" as const, label: "No" },
];
const YES_NO = [
	{ id: "yes" as const, label: "Yes" },
	{ id: "no" as const, label: "No" },
];

const BASIS_LABEL = {
	Calibrated: "Calibrated",
	TeamExperience: "Team experience",
	Derived: "Derived",
};

/// A range as it reads: one number when both ends agree.
function range(s: MigrationSpan): string {
	return s.low === s.high ? `${s.low}` : `${s.low} to ${s.high}`;
}

/// A whole number with thousands separators.
function thousands(n: number): string {
	return n.toLocaleString("en");
}

function asBool(v: "yes" | "no" | null): boolean | null {
	return v == null ? null : v === "yes";
}
function fromBool(v: boolean | null): "yes" | "no" | null {
	return v == null ? null : v ? "yes" : "no";
}

/// Three bands as choices: under, between and over the thresholds.
function bands(b: MigrationCaseBands) {
	return [
		{ id: "small" as const, label: `Under ${thousands(b.medium_from)}` },
		{
			id: "medium" as const,
			label: `${thousands(b.medium_from)} to ${thousands(b.large_over)}`,
		},
		{ id: "large" as const, label: `Over ${thousands(b.large_over)}` },
	];
}

/// Whether an area is in scope as answered: the tier's areas, with those
/// added or removed.
function picked(model: MigrationModelView, answers: MigrationEstimateInput, area: MigrationArea): boolean {
	const tierAreas = model.tiers.find((t) => t.tier === answers.tier)?.areas ?? [];
	return (tierAreas.includes(area) && !answers.remove.includes(area)) || answers.add.includes(area);
}

/// The answers as sent. A count belongs to its area, so it is kept but not
/// sent while that area is unticked; a count would otherwise bring it back.
function sent(model: MigrationModelView, answers: MigrationEstimateInput): MigrationEstimateInput {
	return {
		...answers,
		form_count: picked(model, answers, "M10f") ? answers.form_count : null,
		registry_count: picked(model, answers, "M10r") ? answers.registry_count : null,
	};
}

/// Size the work of moving a client's data into Tamanu (spec MIG).
export default function MigrationEstimator({ model }: { model: MigrationModelView }) {
	const { migrationPath } = useEdition();
	const [route, setRoute] = useState<Route>("bes");
	const [answers, setAnswers] = useState<MigrationEstimateInput>(STARTING_ANSWERS);
	const [engagement, setEngagement] = useState<MigrationImportInput>(NO_PROJECTS);
	const [estimate, setEstimate] = useState<MigrationEstimate | null>(null);
	const [imported, setImported] = useState<MigrationImportEstimate | null>(null);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		if (!migrationPath) return;
		const controller = new AbortController();
		const failed = (e: unknown) => {
			if (controller.signal.aborted) return;
			// A figure that could not be refreshed is not left standing.
			setEstimate(null);
			setImported(null);
			setError(e instanceof Error ? e.message : String(e));
		};
		if (route === "bes") {
			callMigration(migrationPath, "estimate", sent(model, answers), controller.signal)
				.then((e) => {
					setEstimate(e);
					setError(null);
				})
				.catch(failed);
		} else {
			callMigration(migrationPath, "import", engagement, controller.signal)
				.then((e) => {
					setImported(e);
					setError(null);
				})
				.catch(failed);
		}
		return () => controller.abort();
	}, [migrationPath, model, route, answers, engagement]);

	return (
		<div className="calc">
			<h1 className="calc-title">Data migration estimator</h1>
			<main className="calc-main">
				<div className="card">
					<h3 className="qtitle">Who prepares the data</h3>
					<p className="qhelp">
						When the client formats the data into BES templates and does its own mapping, the work is days
						for each program rather than months.
					</p>
					<Segmented
						label="Who prepares the data"
						options={[
							{
								id: "bes" as const,
								label: "BES takes the data out",
							},
							{
								id: "client" as const,
								label: "Client fills BES templates",
							},
						]}
						value={route}
						onChange={setRoute}
					/>
				</div>
				{route === "bes" ? (
					<>
						<BesLedForm
							model={model}
							answers={answers}
							onChange={setAnswers}
							scoped={estimate?.areas.map((a) => a.area) ?? null}
						/>
						{estimate && <EstimateDetail estimate={estimate} />}
					</>
				) : (
					<>
						<ImportForm model={model} engagement={engagement} onChange={setEngagement} />
						{imported && <ImportDetail estimate={imported} />}
					</>
				)}
			</main>

			<aside className="calc-result">
				<div className="card">
					{route === "bes" ? (
						<EstimateSummary estimate={estimate} missing={error} />
					) : (
						<ImportSummary estimate={imported} missing={error} />
					)}
				</div>
			</aside>
		</div>
	);
}

// ── The BES-led route ──────────────────────────────────────────────────────

function BesLedForm({
	model,
	answers,
	onChange,
	scoped,
}: {
	model: MigrationModelView;
	answers: MigrationEstimateInput;
	onChange: (a: MigrationEstimateInput) => void;
	/// The scope the last estimate worked from, once the completion rules ran.
	scoped: MigrationArea[] | null;
}) {
	const set = (patch: Partial<MigrationEstimateInput>) => onChange({ ...answers, ...patch });
	const tierAreas = model.tiers.find((t) => t.tier === answers.tier)?.areas ?? [];
	const tierHint = model.tiers.find((t) => t.tier === answers.tier)?.hint;

	const inScope = (area: MigrationArea) => picked(model, answers, area);
	// Put areas in or out of scope, as a difference from what the tier includes.
	function scope(changes: [MigrationArea, boolean][]) {
		let { add, remove } = answers;
		for (const [area, on] of changes) {
			add = add.filter((a) => a !== area);
			remove = remove.filter((a) => a !== area);
			if (on && !tierAreas.includes(area)) add = [...add, area];
			if (!on && tierAreas.includes(area)) remove = [...remove, area];
		}
		set({ add, remove });
	}

	// One line per area, or per group of alternatives, in the model's order;
	// a group's alternatives run from the cheapest.
	const pickable = model.areas.filter((a) => a.pick != null);
	const lines: {
		key: string;
		label: string;
		members: MigrationModelView["areas"];
	}[] = [];
	for (const a of pickable) {
		if (a.group == null) lines.push({ key: a.area, label: a.pick ?? a.name, members: [a] });
		else if (!lines.some((l) => l.key === a.group))
			lines.push({
				key: a.group,
				label: a.group,
				members: pickable
					.filter((m) => m.group === a.group)
					.sort((x, y) => x.weeks.high - y.weeks.high || x.weeks.low - y.weeks.low),
			});
	}
	// Ticking a group takes the form its tier includes, or else the cheapest.
	function tick(members: MigrationModelView["areas"], on: boolean) {
		const chosen = members.find((m) => tierAreas.includes(m.area)) ?? members[0];
		scope(members.map((m) => [m.area, on && m.area === chosen.area]));
	}

	// Questions follow the scope the estimate works from (spec MIG, Scope).
	const final = scoped ?? model.areas.map((a) => a.area).filter(inScope);

	return (
		<>
			<div className="card">
				<h3 className="qtitle">What the client wants moved</h3>
				<Field label="Closest to what they need">
					<div className="calc-stack">
						<Choice
							label="Closest to what they need"
							options={model.tiers.map((t) => ({
								id: t.tier,
								label: t.label,
							}))}
							value={answers.tier}
							// The areas picked are a difference from a tier, so a new tier starts them again.
							onChange={(tier) => set({ tier, add: [], remove: [] })}
						/>
						<p className="cost-hint">
							{tierHint ??
								`Not chosen: the estimate runs from "${
									model.tiers.find((t) => t.tier === "reference-and-users")?.label ?? ""
								}" alone up to every area.`}
						</p>
					</div>
				</Field>
				{answers.tier != null && (
					<Field label="Areas to move">
						<div className="mig-checks">
							{lines.map(({ key, label, members }) => {
								const on = members.find((m) => inScope(m.area));
								const count =
									on?.area === "M10f"
										? {
												value: answers.form_count,
												set: (form_count: number | null) => set({ form_count }),
											}
										: on?.area === "M10r"
											? {
													value: answers.registry_count,
													set: (registry_count: number | null) => set({ registry_count }),
												}
											: null;
								return (
									<div key={key}>
										<label className="mig-check">
											<input
												type="checkbox"
												checked={on != null}
												onChange={(e) => tick(members, e.target.checked)}
											/>
											<span>{label}</span>
										</label>
										{on && members.length > 1 && (
											<div className="mig-check-detail">
												<Choice
													label={label}
													options={members.map((m) => ({
														id: m.area,
														label: m.pick ?? m.name,
													}))}
													value={on.area}
													// Unticking takes a group out; choosing again keeps the choice.
													onChange={(area) =>
														area && scope(members.map((m) => [m.area, m.area === area]))
													}
												/>
											</div>
										)}
										{count && (
											<div className="mig-check-detail">
												<Count
													label={`How many: ${label}`}
													value={count.value}
													max={model.max_count}
													placeholder="0"
													onChange={count.set}
												/>
											</div>
										)}
									</div>
								);
							})}
						</div>
					</Field>
				)}
			</div>

			<div className="card">
				<h3 className="qtitle">The sites</h3>
				<Field label="Sites going live">
					<Count
						label="Sites going live"
						value={answers.facility_count}
						max={model.max_count}
						placeholder="1"
						onChange={(facility_count) => set({ facility_count })}
					/>
				</Field>
				<Field label="Tamanu already in use there">
					<Choice
						label="Tamanu already in use there"
						options={YES_NO}
						value={fromBool(answers.tamanu_live)}
						onChange={(v) => set({ tamanu_live: asBool(v) })}
					/>
				</Field>
				<Field label="Patients a day at the busiest site">
					<Choice
						label="Patients a day at the busiest site"
						options={bands(model.patients_bands)}
						value={answers.patients_per_day}
						onChange={(patients_per_day) => set({ patients_per_day })}
					/>
				</Field>
				{(final.includes("M4") || final.includes("M4h")) && (
					<Field label="Years of visit history to move">
						<Choice
							label="Years of visit history to move"
							options={bands(model.years_bands)}
							value={answers.history_years}
							onChange={(history_years) => set({ history_years })}
						/>
					</Field>
				)}
				<Field label="Staff to be migrated">
					<div className="calc-stack">
						<Choice
							label="Staff to be migrated"
							options={[
								{
									id: "no" as const,
									label: `Up to ${thousands(model.many_users_over)}`,
								},
								{
									id: "yes" as const,
									label: `Over ${thousands(model.many_users_over)}`,
								},
							]}
							value={fromBool(answers.many_users)}
							onChange={(v) => set({ many_users: asBool(v) })}
						/>
						<p className="cost-hint">Includes past staff, if records are to be attributed to them.</p>
					</div>
				</Field>
			</div>

			<div className="card">
				<h3 className="qtitle">The current system</h3>
				<Field label="What they use now">
					<Choice
						label="What they use now"
						options={[
							{
								id: "single-system" as const,
								label: "One system",
							},
							{
								id: "several-systems" as const,
								label: "Several systems",
							},
							{
								id: "spreadsheets" as const,
								label: "Spreadsheets",
							},
							{ id: "mix" as const, label: "A mix" },
						]}
						value={answers.source_kind}
						onChange={(source_kind) => set({ source_kind })}
					/>
				</Field>
				<Field label="Separate installs">
					<div className="calc-stack">
						<Choice
							label="Separate installs"
							options={[
								{ id: "one" as const, label: "1" },
								{
									id: "some" as const,
									label: `${model.installs_some_from} to ${model.installs_many_from - 1}`,
								},
								{ id: "many" as const, label: `${model.installs_many_from} or more` },
							]}
							value={
								answers.source_instances == null
									? null
									: answers.source_instances >= model.installs_many_from
										? "many"
										: answers.source_instances >= model.installs_some_from
											? "some"
											: "one"
							}
							// Each band is sent as its smallest count.
							onChange={(v) =>
								set({
									source_instances:
										v == null
											? null
											: {
													one: 1,
													some: model.installs_some_from,
													many: model.installs_many_from,
												}[v],
								})
							}
						/>
						<p className="cost-hint">
							The same system installed at different sites, each keeping its own data with no central
							database.
						</p>
					</div>
				</Field>
				{answers.source_kind !== "spreadsheets" && (
					<Field label="Database">
						<Choice
							label="Database"
							options={[
								{
									id: "mysql-or-postgres" as const,
									label: "MySQL or Postgres",
								},
								{
									id: "other" as const,
									label: "Something else",
								},
							]}
							value={answers.source_db}
							onChange={(source_db) => set({ source_db })}
						/>
					</Field>
				)}
				<Field label="An up-to-date data dictionary">
					<Choice
						label="An up-to-date data dictionary"
						options={DOC_LEVELS}
						value={answers.doc_dictionary}
						onChange={(doc_dictionary) => set({ doc_dictionary })}
					/>
				</Field>
				<Field label="Code lists can be exported">
					<Choice
						label="Code lists can be exported"
						options={DOC_LEVELS}
						value={answers.doc_codes}
						onChange={(doc_codes) => set({ doc_codes })}
					/>
				</Field>
				<Field label="Someone who knows how the data is stored">
					<Choice
						label="Someone who knows how the data is stored"
						options={DOC_LEVELS}
						value={answers.doc_contact}
						onChange={(doc_contact) => set({ doc_contact })}
					/>
				</Field>
				{/* Earlier exports stand in for a data dictionary that is not there. */}
				{answers.doc_dictionary !== "good" && (
					<Field label="Earlier exports, with their queries">
						<Choice
							label="Earlier exports, with their queries"
							options={[DOC_LEVELS[0], DOC_LEVELS[2]]}
							value={answers.doc_prior_exports}
							onChange={(doc_prior_exports) => set({ doc_prior_exports })}
						/>
					</Field>
				)}
			</div>

			<div className="card">
				<h3 className="qtitle">The delivery</h3>
				<Field label="Client reviews work">
					<Choice
						label="Client reviews work"
						options={[
							{ id: "fast" as const, label: "Within a week" },
							{ id: "slow" as const, label: "Longer" },
						]}
						value={answers.review_speed}
						onChange={(review_speed) => set({ review_speed })}
					/>
				</Field>
				<Field label="Data people on this">
					<Choice
						label="Data people on this"
						options={[
							{ id: "one" as const, label: "One" },
							{ id: "two" as const, label: "Two" },
							{ id: "three" as const, label: "Three" },
						]}
						value={answers.staffing}
						onChange={(staffing) => set({ staffing })}
					/>
				</Field>
			</div>
		</>
	);
}

function people(n: number | null): string {
	if (n == null) return "Any staffing";
	return n === 1 ? "One data person" : n === 2 ? "Two data people" : "Three data people";
}

function EstimateSummary({ estimate, missing }: { estimate: MigrationEstimate | null; missing: string | null }) {
	if (!estimate) {
		return (
			<div className="cost">
				<span className="stat-k">Timeline</span>
				<span className="cost-annual">{missing ?? "Working it out…"}</span>
			</div>
		);
	}
	const headline = estimate.staffing.find((r) => r.as_answered) ?? estimate.staffing[0];
	return (
		<div className="cost">
			<span className="stat-k">
				Timeline
				{headline.people != null ? `, ${people(headline.people).toLowerCase()}` : ""}
			</span>
			<span className="cost-v">{range(headline.months)} months</span>
			<span className="cost-annual">{range(estimate.person_days)} data team person-days</span>
			{estimate.scoping_person_days && (
				<span className="cost-annual">
					{range(estimate.scoping_person_days)} scoping and reference data person-days
				</span>
			)}
			{estimate.narrowing.length > 0 && (
				<p className="cost-hint">
					{estimate.narrowing.length} unanswered question
					{estimate.narrowing.length === 1 ? "" : "s"} widen this range: see what narrows it most.
				</p>
			)}
			<p className="cost-hint">{estimate.narrows_after}</p>
			{estimate.upper_bound && <p className="cost-callout">Upper bound only: see the flags.</p>}
			<ul className="cost-lines">
				<li>
					<span className="cost-line-label">Confidence</span>
					<span className="cost-line-v">{estimate.confidence}</span>
					<span className="cost-line-detail">How much is known, not how hard it is.</span>
				</li>
				{estimate.staffing.map((r) => (
					<li key={r.people ?? "any"}>
						<span className="cost-line-label">{people(r.people)}</span>
						<span className="cost-line-v">{range(r.months)} months</span>
						{r.people == null && (
							<span className="cost-line-detail">Extra data people do not shorten this scope.</span>
						)}
					</li>
				))}
			</ul>
			<p className="cost-caveat">
				Not a quote or a promise of dates: the clock starts once the prerequisites are in place. Model{" "}
				{estimate.model_version}.
			</p>
		</div>
	);
}

function EstimateDetail({ estimate }: { estimate: MigrationEstimate }) {
	const alt = estimate.documentation_alternatives;
	return (
		<>
			{estimate.narrowing.length > 0 && (
				<div className="card">
					<h3 className="qtitle">What narrows the range most</h3>
					<p className="cost-hint">
						The questions not yet answered, those that move the estimate most first, with the timeline and
						data team person-days under each one's cheapest and costliest answer.
					</p>
					<table className="mig-table mig-narrow">
						<thead>
							<tr>
								<th>Question</th>
								<th>Cheapest answer</th>
								<th>Costliest answer</th>
							</tr>
						</thead>
						<tbody>
							{estimate.narrowing.map((n) => (
								<tr key={n.question}>
									<td>{n.question}</td>
									<td>
										{range(n.cheapest.months)} months
										<span className="mig-sub">{range(n.cheapest.person_days)} person-days</span>
									</td>
									<td>
										{range(n.costliest.months)} months
										<span className="mig-sub">{range(n.costliest.person_days)} person-days</span>
									</td>
								</tr>
							))}
						</tbody>
					</table>
				</div>
			)}

			<div className="card">
				<h3 className="qtitle">Areas in scope</h3>
				<table className="mig-table">
					<thead>
						<tr>
							<th>Area</th>
							<th>Elapsed weeks</th>
							<th>Data person-days</th>
							<th>Basis</th>
						</tr>
					</thead>
					<tbody>
						{estimate.areas.map((a) => (
							<tr key={a.area}>
								<td>
									<span className="mig-code">{a.area}</span> {a.name}
									{a.count != null && ` (${range(a.count)})`}
								</td>
								<td>{range(a.weeks)}</td>
								<td>{range(a.person_days)}</td>
								<td>{BASIS_LABEL[a.basis]}</td>
							</tr>
						))}
					</tbody>
				</table>
				{estimate.scoping_person_days && (
					<p className="cost-hint mig-note">
						M0 and M1 are often led by the Program Manager for smaller deployments, with minor data team
						involvement; if so, only the data team person-days fall to the data team.
					</p>
				)}
			</div>

			{alt && (
				<div className="card">
					<h3 className="qtitle">What the client's documentation is worth</h3>
					<table className="mig-table">
						<thead>
							<tr>
								<th>If</th>
								<th>Timeline (months)</th>
								<th>Data team person-days</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td>Documentation turns out good</td>
								<td>{range(alt.good.months)}</td>
								<td>{range(alt.good.person_days)}</td>
							</tr>
							<tr>
								<td>Nobody can explain the data</td>
								<td>{range(alt.no_contact.months)}</td>
								<td>{range(alt.no_contact.person_days)}</td>
							</tr>
						</tbody>
					</table>
				</div>
			)}

			<div className="card">
				<h3 className="qtitle">Flags</h3>
				<Notes
					items={[
						...estimate.stop_flags.map((text) => ({
							text,
							stop: true,
						})),
						...estimate.flags.map((text) => ({
							text,
							stop: false,
						})),
					]}
					none="Nothing assumed."
				/>
				{estimate.unknown.length > 0 && (
					<p className="cost-hint mig-note">Key answers still unknown: {estimate.unknown.join(", ")}.</p>
				)}
			</div>

			<div className="card">
				<h3 className="qtitle">Workings</h3>
				<Notes
					items={estimate.workings.map((text) => ({
						text,
						stop: false,
					}))}
					none="No adjustments."
					plain
				/>
			</div>

			<div className="card">
				<h3 className="qtitle">Not included</h3>
				<Notes
					items={estimate.not_included.map((text) => ({
						text,
						stop: false,
					}))}
					none=""
					plain
				/>
			</div>
		</>
	);
}

// ── The client-prepared route ──────────────────────────────────────────────

function ImportForm({
	model,
	engagement,
	onChange,
}: {
	model: MigrationModelView;
	engagement: MigrationImportInput;
	onChange: (e: MigrationImportInput) => void;
}) {
	const { projects } = engagement;
	const setProject = (i: number, patch: Partial<MigrationImportProject>) =>
		onChange({
			...engagement,
			projects: projects.map((p, j) => (j === i ? { ...p, ...patch } : p)),
		});
	return (
		<>
			<div className="card">
				<h3 className="qtitle">The engagement</h3>
				<p className="qhelp">
					A program is the data for one health program that the client formats into BES templates and hands
					over for import, for example NCD or antenatal care at the sites that run it. A program usually has
					one registry, which brings in its patients, and several forms; each is its own template and import.
					Data collected with the same tools and handed over together belongs in one program, and programs can
					run side by side.
				</p>
				<Field label="BES import templates already exist">
					<Choice
						label="BES import templates already exist"
						options={YES_NO}
						value={fromBool(engagement.templates_exist)}
						onChange={(v) =>
							onChange({
								...engagement,
								templates_exist: asBool(v),
							})
						}
					/>
				</Field>
			</div>
			{projects.map((p, i) => (
				// Programs carry no name, so their position is their identity.
				<div className="card" key={i}>
					<div className="mig-project-head">
						<h3 className="qtitle">Program {i + 1}</h3>
						{projects.length > 1 && (
							<button
								type="button"
								className="calc-disclose"
								onClick={() =>
									onChange({
										...engagement,
										projects: projects.filter((_, j) => j !== i),
									})
								}
							>
								Remove
							</button>
						)}
					</div>
					<Field label="Has a registry">
						<Choice
							label={`Program ${i + 1} has a registry`}
							options={YES_NO}
							value={fromBool(p.registry)}
							onChange={(v) => setProject(i, { registry: asBool(v) })}
						/>
					</Field>
					<Field label="Forms to import">
						<Count
							label={`Program ${i + 1} forms to import`}
							value={p.forms}
							max={model.max_count}
							onChange={(forms) => setProject(i, { forms: forms ?? 0 })}
						/>
					</Field>
					<Field label="Sites">
						<Count
							label={`Program ${i + 1} sites`}
							value={p.sites}
							max={model.max_count}
							onChange={(sites) => setProject(i, { sites })}
						/>
					</Field>
					<Field label="Cases in total, roughly">
						<Choice
							label={`Program ${i + 1} cases`}
							options={bands(model.case_bands)}
							value={p.cases}
							onChange={(cases) => setProject(i, { cases })}
						/>
					</Field>
				</div>
			))}
			{projects.length < model.max_projects && (
				<button
					type="button"
					className="topbar-new mig-add"
					onClick={() =>
						onChange({
							...engagement,
							projects: [...projects, NEW_PROJECT],
						})
					}
				>
					Add a program
				</button>
			)}
		</>
	);
}

function ImportSummary({ estimate, missing }: { estimate: MigrationImportEstimate | null; missing: string | null }) {
	if (!estimate) {
		return (
			<div className="cost">
				<span className="stat-k">Data team days</span>
				<span className="cost-annual">{missing ?? "Working it out…"}</span>
			</div>
		);
	}
	return (
		<div className="cost">
			<span className="stat-k">Data team days</span>
			<span className="cost-v">{range(estimate.total_days)} days</span>
			<ul className="cost-lines">
				{estimate.projects.map((p) => (
					<li key={p.number}>
						<span className="cost-line-label">Program {p.number}</span>
						<span className="cost-line-v">{range(p.days)} days</span>
						<span className="cost-line-detail">
							About {range(p.weeks)} weeks once the formatted data arrives
						</span>
					</li>
				))}
				{estimate.templates_days != null && (
					<li>
						<span className="cost-line-label">Import templates</span>
						<span className="cost-line-v">
							{range(estimate.templates_days)} day
							{estimate.templates_days.high === 1 ? "" : "s"}
						</span>
						<span className="cost-line-detail">Once for the engagement</span>
					</li>
				)}
			</ul>
			<p className="cost-caveat">
				The client's own formatting time is not included. Model {estimate.model_version}.
			</p>
		</div>
	);
}

function ImportDetail({ estimate }: { estimate: MigrationImportEstimate }) {
	return (
		<>
			<div className="card">
				<h3 className="qtitle">Flags</h3>
				<Notes
					items={estimate.flags.map((text) => ({
						text,
						stop: false,
					}))}
					none="Nothing assumed."
				/>
			</div>
			<div className="card">
				<h3 className="qtitle">Workings</h3>
				<Notes
					items={estimate.workings.map((text) => ({
						text,
						stop: false,
					}))}
					none=""
					plain
				/>
			</div>
			<div className="card">
				<h3 className="qtitle">Not included</h3>
				<Notes
					items={estimate.not_included.map((text) => ({
						text,
						stop: false,
					}))}
					none=""
					plain
				/>
			</div>
		</>
	);
}

// ── Controls ───────────────────────────────────────────────────────────────

/// A row of options where choosing the selected one again clears it, which is
/// how an answer goes back to unknown.
function Choice<T extends string>({
	options,
	value,
	onChange,
	label,
}: {
	options: { id: T; label: string }[];
	value: T | null;
	onChange: (v: T | null) => void;
	label: string;
}) {
	return (
		<div className={`bandrow${options.length === 3 ? " bandrow-three" : ""}`} role="group" aria-label={label}>
			{options.map((o) => (
				<button
					type="button"
					key={o.id}
					aria-pressed={value === o.id}
					className={`band${value === o.id ? " on" : ""}`}
					onClick={() => onChange(value === o.id ? null : o.id)}
				>
					{o.label}
				</button>
			))}
		</div>
	);
}

/// A whole number, or nothing for unknown.
function Count({
	value,
	onChange,
	label,
	max,
	placeholder = "Not known",
}: {
	value: number | null;
	onChange: (v: number | null) => void;
	label: string;
	max: number;
	placeholder?: string;
}) {
	return (
		<input
			className="mig-count"
			type="number"
			inputMode="numeric"
			aria-label={label}
			placeholder={placeholder}
			min={0}
			max={max}
			step={1}
			value={value ?? ""}
			onChange={(e) => {
				const n = e.target.value === "" ? Number.NaN : Math.floor(Number(e.target.value));
				onChange(Number.isFinite(n) && n >= 0 ? Math.min(n, max) : null);
			}}
		/>
	);
}

function Notes({
	items,
	none,
	plain = false,
}: {
	items: { text: string; stop: boolean }[];
	none: string;
	plain?: boolean;
}) {
	if (items.length === 0) return none ? <p className="cost-hint">{none}</p> : null;
	return (
		<div className="flags">
			{items.map((item) => (
				<div key={item.text} className={`flag${item.stop ? " flag-block" : ""}`}>
					<span className={`flag-dot${plain ? " mig-dot-plain" : ""}`} />
					<span>{item.text}</span>
				</div>
			))}
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
