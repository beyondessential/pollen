//! The BES-led route of the effort model: scope from a tier and the areas
//! picked, ranges adjusted by what is known of the source, then a timeline,
//! person-days and a confidence (spec MIG). Every number comes from the
//! configuration; this applies it.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::client_prepared::CaseBand;
use super::config::{
	Adjust, Area, Basis, CaseBands, Combine, DocFactor, MigrationConfig, Span, Tier,
};
use super::{fmt_num, round_half, round_whole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
	SingleSystem,
	SeveralSystems,
	Spreadsheets,
	Mix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SourceDb {
	/// MySQL or Postgres, the databases the migration tooling reads directly.
	#[serde(alias = "mysql")]
	MysqlOrPostgres,
	Other,
}

/// An answer to one of the documentation questions, and the rating of the four
/// together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DocLevel {
	Good,
	Partial,
	None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewSpeed {
	Fast,
	Slow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Staffing {
	One,
	Two,
	Three,
}

impl Staffing {
	fn people(self) -> u32 {
		match self {
			Staffing::One => 1,
			Staffing::Two => 2,
			Staffing::Three => 3,
		}
	}
}

/// The answers a BES-led estimate is worked out from. Every one can be left
/// out, which reads as unknown.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct EstimateInput {
	#[serde(default)]
	pub tier: Option<Tier>,
	/// Areas added to the tier's.
	#[serde(default)]
	pub add: Vec<Area>,
	/// Areas taken out of the tier's.
	#[serde(default)]
	pub remove: Vec<Area>,
	#[serde(default)]
	pub form_count: Option<u32>,
	#[serde(default)]
	pub registry_count: Option<u32>,
	/// Sites going live, each with its own cutover.
	#[serde(default)]
	pub facility_count: Option<u32>,
	#[serde(default)]
	pub tamanu_live: Option<bool>,
	#[serde(default)]
	pub source_kind: Option<SourceKind>,
	/// Separate copies of the source system, read in bands: one, a few, and
	/// many.
	#[serde(default)]
	pub source_instances: Option<u32>,
	#[serde(default)]
	pub source_db: Option<SourceDb>,
	#[serde(default)]
	pub doc_dictionary: Option<DocLevel>,
	#[serde(default)]
	pub doc_codes: Option<DocLevel>,
	#[serde(default)]
	pub doc_contact: Option<DocLevel>,
	#[serde(default)]
	pub doc_prior_exports: Option<DocLevel>,
	/// Patients seen a day in total across all sites, as a band.
	#[serde(default)]
	pub patients_per_day: Option<CaseBand>,
	/// Years of visit history to migrate, as a band. Only read when visit
	/// history is in scope.
	#[serde(default)]
	pub history_years: Option<CaseBand>,
	/// Whether more staff are to be migrated than `users.many_over`, past
	/// staff included when records are attributed to them.
	#[serde(default)]
	pub many_users: Option<bool>,
	#[serde(default)]
	pub review_speed: Option<ReviewSpeed>,
	#[serde(default)]
	pub staffing: Option<Staffing>,
}

/// How much is known about the migration, not how hard it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub enum ConfidenceLevel {
	Low,
	Medium,
	High,
}

/// One area in scope, with its figures as displayed.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AreaLine {
	pub area: Area,
	pub name: String,
	/// How many of it, for forms and registries: a range when the count was
	/// not answered.
	pub count: Option<Span>,
	/// Elapsed weeks, to the nearest half.
	pub weeks: Span,
	pub person_days: Span,
	pub basis: Basis,
}

/// The timeline for a number of data people.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StaffingRow {
	/// The data people; none when the timeline is the same for any staffing.
	pub people: Option<u32>,
	/// Whether this is the staffing that was answered, or assumed.
	pub as_answered: bool,
	/// Months, to the nearest half.
	pub months: Span,
}

/// The headline figures under other answers.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Alternative {
	pub months: Span,
	pub person_days: Span,
	pub scoping_person_days: Option<Span>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DocumentationAlternatives {
	/// If documentation turns out good.
	pub good: Alternative,
	/// If there turns out to be no knowledgeable contact.
	pub no_contact: Alternative,
}

/// An unanswered question, and the range with its cheapest and costliest
/// answer.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Narrowing {
	/// The question as the form asks it.
	pub question: String,
	pub cheapest: Alternative,
	pub costliest: Alternative,
}

/// A BES-led estimate, as displayed.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Estimate {
	pub model_version: String,
	/// The tier chosen. When none is, the range runs from the smallest scope
	/// to the largest.
	pub tier: Option<Tier>,
	pub confidence: ConfidenceLevel,
	pub documentation: DocLevel,
	/// The figures are an upper bound (a spreadsheet-only source).
	pub upper_bound: bool,
	pub areas: Vec<AreaLine>,
	pub staffing: Vec<StaffingRow>,
	/// Data team person-days, with the review allowance.
	pub person_days: Span,
	/// M0 and M1 person-days, reported on their own for the smaller tiers.
	pub scoping_person_days: Option<Span>,
	pub documentation_alternatives: Option<DocumentationAlternatives>,
	/// The unanswered questions that move the range, those that move it most
	/// first.
	pub narrowing: Vec<Narrowing>,
	/// Each adjustment applied, and how the timeline is put together.
	pub workings: Vec<String>,
	/// Conditions the Data Lead weighs before the estimate goes further.
	pub stop_flags: Vec<String>,
	/// Each assumption made and each change to the scope.
	pub flags: Vec<String>,
	/// The key answers still unknown.
	pub unknown: Vec<String>,
	pub not_included: Vec<String>,
	/// Why the range is wide, and when a better estimate can be given.
	pub narrows_after: String,
}

/// An area in scope while its range is worked out.
struct Line {
	area: Area,
	low: f64,
	high: f64,
	count: Option<u32>,
}

/// An estimate before rounding for display.
struct Worked {
	tier: Tier,
	lines: Vec<Line>,
	doc: DocLevel,
	spreadsheet: bool,
	people: u32,
	/// What the answers add to the timeline and the person-days, whatever the
	/// staffing, each its own range.
	adjusts: Vec<Adjust>,
	/// Whether the client is slow to review, which adds to the high end of the
	/// timeline only.
	slow_review: bool,
	data: Span,
	scoping: Option<Span>,
	confidence: ConfidenceLevel,
	workings: Vec<String>,
	flags: Vec<String>,
	unknown: Vec<String>,
}

fn list(areas: &[Area]) -> String {
	areas
		.iter()
		.map(|a| a.code())
		.collect::<Vec<_>>()
		.join(", ")
}

impl MigrationConfig {
	fn cap(&self, n: u32) -> u32 {
		n.min(self.max_count)
	}

	/// The scope: the tier's areas, adjusted by what was added and removed, and
	/// completed by what each area depends on (spec MIG, Scope).
	fn scope(
		&self,
		a: &EstimateInput,
		tier: Tier,
		flags: &mut Vec<String>,
	) -> (Vec<Area>, u32, u32) {
		let mut areas = self.tier(tier).areas.clone();
		for &code in &a.add {
			if !areas.contains(&code) {
				areas.push(code);
			}
		}
		areas.retain(|c| !a.remove.contains(c));

		// Forms and registries: a count adds the area, and no count (or zero)
		// leaves it out, since often there are none to migrate. Settled before
		// the dependencies below, so an area added by its count still brings in
		// what it needs.
		let mut counted = |area: Area, given: Option<u32>, what: &str| {
			let count = given.map_or(0, |n| self.cap(n));
			if count > 0 && !areas.contains(&area) {
				areas.push(area);
			}
			if count == 0 && areas.contains(&area) {
				areas.retain(|c| *c != area);
				flags.push(format!(
					"{what} ticked with no count: none counted, so left out."
				));
			}
			count
		};
		let forms = counted(Area::M10f, a.form_count, "Clinical forms");
		let registries = counted(Area::M10r, a.registry_count, "Disease or program forms");

		// Every area needs the scoping and reference data; any clinical area
		// also needs users.
		let clinical = areas
			.iter()
			.any(|c| !matches!(c, Area::M0 | Area::M1 | Area::M2));
		for code in [Area::M0, Area::M1, Area::M2] {
			if !areas.contains(&code) && (code != Area::M2 || clinical) {
				areas.push(code);
				if a.remove.contains(&code) {
					flags.push(format!(
						"{} cannot be removed: it is needed for any migration{}",
						code.code(),
						if code == Area::M2 {
							" moving clinical data."
						} else {
							"."
						}
					));
				}
			}
		}

		if areas.contains(&Area::M6) && !areas.contains(&Area::M5) {
			areas.push(Area::M5);
			flags.push("MAR (M6) needs medications: added M5.".into());
		}
		let needs: Vec<Area> = areas
			.iter()
			.copied()
			.filter(|c| c.needs_encounters())
			.collect();
		let later: Vec<Area> = areas
			.iter()
			.copied()
			.filter(|c| c.after_encounters())
			.collect();
		if !later.is_empty() && areas.contains(&Area::M4h) {
			areas.retain(|c| *c != Area::M4h);
			if !areas.contains(&Area::M4) {
				areas.push(Area::M4);
				flags.push(format!(
					"Encounters raised from patient history (M4h) to full encounters (M4) because {} {} in scope.",
					list(&later),
					if later.len() == 1 { "is" } else { "are" }
				));
			}
		}
		if !needs.is_empty() && !areas.contains(&Area::M4) {
			areas.push(Area::M4);
			flags.push(format!(
				"Added encounters (M4): Tamanu records {} against encounters.",
				list(&needs)
			));
		}
		if areas.contains(&Area::M4) {
			areas.retain(|c| *c != Area::M4h);
		}
		let encounters = areas.iter().any(|c| matches!(c, Area::M4 | Area::M4h));
		let patients = areas.contains(&Area::M3);
		if encounters && !patients {
			areas.push(Area::M3);
			flags.push("Encounters need patients: added patients (M3).".into());
		}
		if areas.contains(&Area::M11f) {
			areas.retain(|c| *c != Area::M11);
		}
		if areas.contains(&Area::M12f) {
			areas.retain(|c| *c != Area::M12b);
		}

		areas.sort();
		areas.dedup();
		(areas, forms, registries)
	}

	/// Ranges added together (spec MIG, Timeline): stacked end to end, or as
	/// independent estimates, each spread around its midpoint and the spreads
	/// combined as the root of the sum of their squares.
	fn add_up(&self, parts: &[Span]) -> Span {
		match self.combine {
			Combine::Stacked => parts.iter().fold(Span::ZERO, |t, p| Span {
				low: t.low + p.low,
				high: t.high + p.high,
			}),
			Combine::Independent => {
				let mid: f64 = parts.iter().map(|p| (p.low + p.high) / 2.0).sum();
				let spread = parts
					.iter()
					.map(|p| ((p.high - p.low) / 2.0).powi(2))
					.sum::<f64>()
					.sqrt();
				Span {
					low: mid - spread,
					high: mid + spread,
				}
			}
		}
	}

	/// An adjustment an answer adds, with its working.
	fn push(&self, by: Adjust, why: &str, adjusts: &mut Vec<Adjust>, workings: &mut Vec<String>) {
		if by.weeks.high <= 0.0 && by.days.high <= 0.0 {
			return;
		}
		workings.push(format!(
			"{why}: added {} and {}.",
			amount(by.weeks, "week"),
			amount(by.days, "person-day")
		));
		adjusts.push(by);
	}

	/// What each documentation answer multiplies every area by (spec MIG,
	/// Ranges).
	fn document_each(
		&self,
		a: &EstimateInput,
		lines: &mut [Line],
		flags: &mut Vec<String>,
		workings: &mut Vec<String>,
	) -> DocLevel {
		let mut known = |answer: Option<DocLevel>, what: &str| {
			answer.unwrap_or_else(|| {
				flags.push(format!("{what} unknown: treated as Partly."));
				DocLevel::Partial
			})
		};
		let contact = known(a.doc_contact, "Knowledgeable contact");
		let codes = known(a.doc_codes, "Code lists");
		let dictionary = known(a.doc_dictionary, "Data dictionary");

		let d = &self.documentation;
		let answers: [(DocLevel, DocFactor, &str, &str); 3] = [
			(
				contact,
				d.contact,
				"Someone partly knows how the data is stored",
				"Nobody knows how the data is stored",
			),
			(
				codes,
				d.codes,
				"Code lists can partly be exported",
				"Code lists cannot be exported",
			),
			(
				dictionary,
				d.dictionary,
				"Data dictionary partly there or out of date",
				"No data dictionary",
			),
		];
		let mut by: Vec<(f64, &str)> = answers
			.into_iter()
			.filter_map(|(level, on, partial, none)| match level {
				DocLevel::Good => None,
				DocLevel::Partial => Some((on.partial, partial)),
				DocLevel::None => Some((on.none, none)),
			})
			.collect();
		// Earlier exports stand in for a data dictionary that is not there.
		if dictionary != DocLevel::Good {
			match a.doc_prior_exports {
				Some(DocLevel::Good) => {}
				Some(_) => by.push((d.no_prior_exports, "No earlier exports with their queries")),
				None => flags.push("Earlier exports unknown: nothing added for them.".into()),
			}
		}
		for (factor, why) in by {
			if factor == 1.0 {
				continue;
			}
			for l in lines.iter_mut() {
				l.low *= factor;
				l.high *= factor;
			}
			workings.push(format!("{why}: every area x{}.", fmt_num(factor)));
		}

		if [a.doc_contact, a.doc_codes, a.doc_dictionary]
			.iter()
			.all(|v| *v == Some(DocLevel::Good))
		{
			DocLevel::Good
		} else if contact == DocLevel::None {
			DocLevel::None
		} else {
			DocLevel::Partial
		}
	}

	/// The timeline in weeks for a number of data people (spec MIG, Timeline).
	fn timeline(&self, w: &Worked, people: u32) -> Span {
		let n = f64::from(people);
		let of = |area: Area| {
			w.lines.iter().find(|l| l.area == area).map(|l| Span {
				low: l.low,
				high: l.high,
			})
		};
		let mut parts = Vec::new();
		// M1 and M2 run during M0.
		let (m0, m1) = (of(Area::M0), of(Area::M1));
		let end = |s: Option<Span>, pick: fn(Span) -> f64| s.map_or(0.0, pick);
		parts.push(Span {
			low: end(m0, |s| s.low).max(end(m1, |s| s.low)),
			high: end(m0, |s| s.high).max(end(m1, |s| s.high)),
		});
		parts.extend([Area::M3, Area::M4, Area::M4h].into_iter().filter_map(of));
		let after: Vec<&Line> = w
			.lines
			.iter()
			.filter(|l| l.area.after_encounters())
			.collect();
		if !after.is_empty() {
			let mut shared = match self.combine {
				Combine::Stacked => Span {
					low: after.iter().map(|l| l.low).sum::<f64>() / n,
					high: after.iter().map(|l| l.high).sum::<f64>() / n,
				},
				Combine::Independent => self.add_up(
					&after
						.iter()
						.map(|l| Span {
							low: l.low / n,
							high: l.high / n,
						})
						.collect::<Vec<_>>(),
				),
			};
			// Forms and registries can be split between people; no other area
			// can finish sooner than it takes one person.
			for l in after.iter().filter(|l| !l.area.counted()) {
				shared.low = shared.low.max(l.low);
				shared.high = shared.high.max(l.high);
			}
			parts.push(shared);
		}
		// The first site's cutover; what the answers add, extra sites included.
		let cutover = self.sites.cutover_weeks;
		parts.push(Span {
			low: cutover,
			high: cutover,
		});
		parts.extend(w.adjusts.iter().map(|a| a.weeks));
		let mut total = self.add_up(&parts);
		// Waiting on the client's reviews lengthens the worst case only.
		if w.slow_review {
			total.high += self.slow_review_weeks;
		}
		total
	}

	fn months(&self, weeks: Span) -> Span {
		weeks.map(|w| round_half(w / self.timeline.weeks_per_month))
	}

	fn share(&self, area: Area) -> f64 {
		self.effort.days_per_week
			* if area == Area::M0 {
				self.effort.m0_share
			} else {
				self.effort.area_share
			}
	}

	fn work(&self, a: &EstimateInput) -> Worked {
		let mut flags = Vec::new();
		let mut workings = Vec::new();

		let tier_known = a.tier.is_some();
		let tier = a.tier.unwrap_or(self.unsure_tier);
		if !tier_known {
			flags.push(format!(
				"Tier not chosen: started from {}.",
				self.tier(tier).label.to_lowercase()
			));
		}
		let (areas, forms, registries) = self.scope(a, tier, &mut flags);

		let mut lines: Vec<Line> = areas
			.iter()
			.map(|&area| {
				let mut weeks = if area == Area::M0 && tier == Tier::FullEhr {
					self.m0_full_ehr.weeks
				} else {
					self.area(area).weeks
				};
				// Staff to be migrated set the users area; unknown, it keeps
				// its full range.
				if area == Area::M2
					&& let Some(many) = a.many_users
				{
					let w = if many {
						self.users.many_weeks
					} else {
						self.users.weeks
					};
					weeks = Span { low: w, high: w };
				}
				let count = match area {
					Area::M10f => Some(forms),
					Area::M10r => Some(registries),
					_ => None,
				};
				let times = f64::from(count.unwrap_or(1));
				Line {
					area,
					low: weeks.low * times,
					high: weeks.high * times,
					count,
				}
			})
			.collect();
		if let Some(many) = a.many_users
			&& lines.iter().any(|l| l.area == Area::M2)
		{
			let u = &self.users;
			workings.push(if many {
				format!(
					"Over {} staff to be migrated: M2 takes {}.",
					u.many_over,
					weeks(u.many_weeks)
				)
			} else {
				format!(
					"Up to {} staff to be migrated: M2 takes {}.",
					u.many_over,
					weeks(u.weeks)
				)
			});
		}

		// Documentation.
		let mut adjusts: Vec<Adjust> = Vec::new();
		let doc = self.document_each(a, &mut lines, &mut flags, &mut workings);

		// Source.
		let spreadsheet = a.source_kind == Some(SourceKind::Spreadsheets);
		if !spreadsheet && a.source_db == Some(SourceDb::Other) {
			self.push(
				self.source.other_db,
				"Source database not MySQL or Postgres",
				&mut adjusts,
				&mut workings,
			);
		}
		if matches!(
			a.source_kind,
			Some(SourceKind::SeveralSystems | SourceKind::Mix)
		) {
			self.push(
				self.source.several,
				"Several systems or a mix",
				&mut adjusts,
				&mut workings,
			);
			flags.push(
				"Several systems or a mix including spreadsheets: a template-based route may suit part of the source."
					.into(),
			);
		}
		if spreadsheet {
			flags.insert(
				0,
				"Spreadsheet-only source: the calibrated figures come from a database EHR and are an upper bound here. The Data Lead should estimate the template-based route before this goes further."
					.into(),
			);
		}
		let instances = a.source_instances.map_or(1, |n| self.cap(n).max(1));
		let i = &self.source.installs;
		if instances >= i.many_from {
			self.push(
				i.many,
				&format!("{} or more separate installs", i.many_from),
				&mut adjusts,
				&mut workings,
			);
		} else if instances >= i.some_from {
			self.push(
				i.some,
				&format!("{} to {} separate installs", i.some_from, i.many_from - 1),
				&mut adjusts,
				&mut workings,
			);
		}

		// Size: the patients a day, then the years of history when visits are
		// in scope, multiply M1 and every area from M3 onwards.
		let sized = |l: &Line| !matches!(l.area, Area::M0 | Area::M2);
		let s = &self.size;
		let history = lines.iter().any(|l| matches!(l.area, Area::M4 | Area::M4h));
		let mut factor = 1.0;
		let mut basis = Vec::new();
		let mut by_band = |band: Option<CaseBand>,
		                   factors: [f64; 3],
		                   bands: CaseBands,
		                   what: &str,
		                   flags: &mut Vec<String>| match band {
			None => flags.push(format!(
				"{} unknown: no size adjustment for them.",
				capitalise(what)
			)),
			Some(band) => {
				let f = factors[band_index(band)];
				if f != 1.0 {
					factor *= f;
					basis.push(format!(
						"x{} for {}",
						fmt_num(f),
						band_label(band, bands, what)
					));
				}
			}
		};
		by_band(
			a.patients_per_day,
			s.patients_factor,
			s.patients,
			"patients a day",
			&mut flags,
		);
		if history {
			by_band(
				a.history_years,
				s.years_factor,
				s.years,
				"years of history",
				&mut flags,
			);
		}
		if factor != 1.0 {
			for l in lines.iter_mut().filter(|l| sized(l)) {
				l.low *= factor;
				l.high *= factor;
			}
			workings.push(format!(
				"Size: {}, so x{} on M1 and M3 onwards.",
				basis.join(" then "),
				fmt_num((factor * 1000.0).round() / 1000.0)
			));
		}

		let slow_review = a.review_speed == Some(ReviewSpeed::Slow);
		if slow_review {
			workings.push(format!(
				"Client slow to review: added {} to the high end of the timeline.",
				weeks(self.slow_review_weeks)
			));
		}

		// Sites and Tamanu already in use.
		let people = a
			.staffing
			.map_or(self.timeline.default_people, Staffing::people);
		let facilities = match a.facility_count.map(|n| self.cap(n)) {
			Some(n) if n > 0 => n,
			_ => {
				flags.push("Number of sites not answered: taken as 1 (one cutover).".into());
				1
			}
		};
		if facilities > 1 {
			let more = facilities - 1;
			self.push(
				self.sites.extra.times(f64::from(more)),
				&format!(
					"{more} more site{} going live",
					if more == 1 { "" } else { "s" }
				),
				&mut adjusts,
				&mut workings,
			);
		}
		if a.tamanu_live == Some(true) {
			self.push(
				self.tamanu_live,
				"Tamanu already in use: checking for conflicts with records entered since go-live",
				&mut adjusts,
				&mut workings,
			);
		}

		// Person-days. For the smaller tiers, M0 and M1 are reported on their
		// own, since they are often led by Program Managers.
		let split = tier != Tier::FullEhr;
		let e = &self.effort;
		let mut data_parts = Vec::new();
		let mut scoping_parts = Vec::new();
		for l in &lines {
			let share = self.share(l.area);
			let part = Span {
				low: l.low * share,
				high: l.high * share,
			};
			if split && matches!(l.area, Area::M0 | Area::M1) {
				scoping_parts.push(part);
			} else {
				data_parts.push(part);
			}
		}
		data_parts.push(self.sites.cutover_days);
		data_parts.extend(adjusts.iter().map(|a| a.days));
		// Each data person beyond the first adds to the effort of keeping the
		// work aligned.
		let align = (1.0 + e.alignment_per_extra_person).powi(people as i32 - 1);
		if align != 1.0 {
			workings.push(format!(
				"{people} data people: person-days x{} for keeping the work aligned.",
				fmt_num((align * 10000.0).round() / 10000.0)
			));
		}
		let data = self
			.add_up(&data_parts)
			.map(|d| d * (1.0 + e.review_allowance) * align);
		let scoping = self.add_up(&scoping_parts).map(|d| d * align);

		// Confidence.
		let mut unknown: Vec<String> = Vec::new();
		let mut miss = |missing: bool, what: &str| {
			if missing {
				unknown.push(what.to_string());
			}
		};
		miss(!tier_known, "tier");
		miss(!tier_known && a.add.is_empty(), "areas");
		miss(a.source_kind.is_none(), "kind of source");
		miss(
			a.source_instances.is_none(),
			"separate installs of the system",
		);
		miss(!spreadsheet && a.source_db.is_none(), "source database");
		miss(a.doc_dictionary.is_none(), "data dictionary");
		miss(a.doc_codes.is_none(), "code lists");
		miss(a.doc_contact.is_none(), "knowledgeable contact");
		miss(
			a.doc_dictionary != Some(DocLevel::Good) && a.doc_prior_exports.is_none(),
			"prior exports",
		);
		miss(a.patients_per_day.is_none(), "patients per day");
		miss(history && a.history_years.is_none(), "years of history");
		miss(a.many_users.is_none(), "staff to be migrated");
		miss(a.review_speed.is_none(), "review speed");
		let derived = lines
			.iter()
			.any(|l| self.area(l.area).basis == Basis::Derived);
		let low = !tier_known
			|| a.source_kind.is_none()
			|| (doc == DocLevel::None && a.doc_contact == Some(DocLevel::None))
			|| a.patients_per_day.is_none()
			|| unknown.len() >= self.confidence.low_from_unknowns;
		let confidence = if low {
			ConfidenceLevel::Low
		} else if doc == DocLevel::Good
			&& unknown.len() <= self.confidence.high_max_unknowns
			&& !derived
		{
			ConfidenceLevel::High
		} else {
			ConfidenceLevel::Medium
		};

		Worked {
			tier,
			lines,
			doc,
			spreadsheet,
			people,
			adjusts,
			slow_review,
			data,
			scoping: split.then_some(scoping),
			confidence,
			workings,
			flags,
			unknown,
		}
	}

	/// Whether an area is in scope as answered, before the completion rules:
	/// the tier's areas, with those added or removed.
	fn picked(&self, a: &EstimateInput, area: Area) -> bool {
		a.add.contains(&area)
			|| a.tier
				.is_some_and(|t| self.tier(t).areas.contains(&area) && !a.remove.contains(&area))
	}

	/// The answers with every unanswered question answered for one end of the
	/// range (spec MIG, Ranges).
	fn answered(&self, a: &EstimateInput, end: End) -> EstimateInput {
		let mut b = a.clone();
		for q in questions() {
			if (q.open)(self, &b) {
				(q.fill)(self, &mut b, end);
			}
		}
		b
	}

	/// The headline figures for a set of answers.
	fn alternative(&self, a: &EstimateInput, ranged: bool) -> Alternative {
		let e = self.present(a, ranged, false);
		let headline = e
			.staffing
			.iter()
			.find(|r| r.as_answered)
			.unwrap_or(&e.staffing[0]);
		Alternative {
			months: headline.months,
			person_days: e.person_days,
			scoping_person_days: e.scoping_person_days,
		}
	}

	/// How the timeline is put together, for the workings.
	fn timeline_working(&self, w: &Worked) -> String {
		let has = |area: Area| w.lines.iter().any(|l| l.area == area);
		let mut sequence = vec!["the longer of M0 and M1 (M2 runs alongside)".to_string()];
		sequence.extend(
			[Area::M3, Area::M4, Area::M4h]
				.into_iter()
				.filter(|a| has(*a))
				.map(Area::code),
		);
		if w.lines.iter().any(|l| l.area.after_encounters()) {
			sequence.push("the remaining areas shared between the data people".into());
		}
		format!(
			"Timeline: {}, plus {} for the first site's cutover and what the answers add.",
			sequence.join(", then "),
			weeks(self.sites.cutover_weeks)
		)
	}

	/// Estimate a BES-led migration from its answers. Each unanswered question
	/// is priced at its cheapest answer for the low end of the range and its
	/// costliest for the high end, so the range narrows as questions are
	/// answered (spec MIG, Ranges).
	pub fn estimate(&self, a: &EstimateInput) -> Estimate {
		self.present(a, true, true)
	}

	/// Estimate with each unknown taken at its configured assumption instead:
	/// the reference model's behaviour. It works out each end of a range, and
	/// the reference cases pin it.
	pub fn estimate_assuming(&self, a: &EstimateInput) -> Estimate {
		self.present(a, false, true)
	}

	fn present(&self, a: &EstimateInput, ranged: bool, extras: bool) -> Estimate {
		let given = self.work(a);
		let open: Vec<&'static str> = if ranged {
			questions()
				.into_iter()
				.filter(|q| (q.open)(self, a))
				.map(|q| q.label)
				.collect()
		} else {
			Vec::new()
		};
		let ends = (!open.is_empty()).then(|| {
			(
				self.work(&self.answered(a, End::Low)),
				self.work(&self.answered(a, End::High)),
			)
		});
		let (lo, hi) = match &ends {
			Some((lo, hi)) => (lo, hi),
			None => (&given, &given),
		};

		// Each area from its low end's figures to its high end's; an area in
		// scope at the high end only starts from nothing.
		let mut codes: Vec<Area> = lo.lines.iter().chain(&hi.lines).map(|l| l.area).collect();
		codes.sort();
		codes.dedup();
		let line = |w: &Worked, area: Area| {
			w.lines
				.iter()
				.find(|l| l.area == area)
				.map(|l| (l.low, l.high, l.count))
		};
		let areas: Vec<AreaLine> = codes
			.into_iter()
			.map(|area| {
				let (l, h) = (line(lo, area), line(hi, area));
				let low = l.map_or(0.0, |l| l.0);
				let high = h.or(l).map_or(0.0, |h| h.1);
				let count = match (l.and_then(|l| l.2), h.and_then(|h| h.2)) {
					(None, None) => None,
					(lc, hc) => Some(Span {
						low: f64::from(lc.unwrap_or(0)),
						high: f64::from(hc.or(lc).unwrap_or(0)),
					}),
				};
				let tier = if h.is_some() { hi.tier } else { lo.tier };
				let (row, share) = (self.area(area), self.share(area));
				AreaLine {
					area,
					name: row.name.clone(),
					count,
					weeks: Span {
						low: round_half(low),
						high: round_half(high),
					},
					person_days: Span {
						low: round_whole(low * share),
						high: round_whole(high * share),
					},
					basis: if area == Area::M0 && tier == Tier::FullEhr {
						self.m0_full_ehr.basis
					} else {
						row.basis
					},
				}
			})
			.collect();

		// Two data people first, as the base case, then three, then one.
		let weeks_for = |n| Span {
			low: self.timeline(lo, n).low,
			high: self.timeline(hi, n).high,
		};
		let base = weeks_for(2);
		let same = |n| {
			let t: Span = weeks_for(n);
			(t.low - base.low).abs() < 1e-9 && (t.high - base.high).abs() < 1e-9
		};
		let staffing = if same(1) && same(3) {
			vec![StaffingRow {
				people: None,
				as_answered: true,
				months: self.months(base),
			}]
		} else {
			[2, 3, 1]
				.into_iter()
				.map(|n| StaffingRow {
					people: Some(n),
					as_answered: n == given.people,
					months: self.months(weeks_for(n)),
				})
				.collect()
		};

		// Scoping is reported on its own only when both ends report it; an end
		// that does is otherwise folded back in, with the review allowance.
		let scoping = match (lo.scoping, hi.scoping) {
			(Some(l), Some(h)) => Some(Span {
				low: l.low,
				high: h.high,
			}),
			_ => None,
		};
		let total = |w: &Worked| match (scoping, w.scoping) {
			(None, Some(s)) => {
				let s = s.map(|d| d * (1.0 + self.effort.review_allowance));
				Span {
					low: w.data.low + s.low,
					high: w.data.high + s.high,
				}
			}
			_ => w.data,
		};
		let person_days = Span {
			low: total(lo).low,
			high: total(hi).high,
		};

		// Workings shared by both ends read plainly; the rest say which end.
		let with_timeline = |w: &Worked| {
			let mut v = w.workings.clone();
			v.push(self.timeline_working(w));
			v
		};
		let (low_workings, high_workings) = (with_timeline(lo), with_timeline(hi));
		let mut workings = Vec::new();
		if !open.is_empty() {
			workings.push(format!(
				"Not answered, so the low end takes the cheapest answer and the high end the costliest: {}.",
				open.join("; ")
			));
		}
		for w in &high_workings {
			workings.push(if low_workings.contains(w) {
				w.clone()
			} else {
				format!("High end: {w}")
			});
		}
		for w in &low_workings {
			if !high_workings.contains(w) {
				workings.push(format!("Low end: {w}"));
			}
		}
		// Flags likewise: one raised at one end only says which.
		let mut flags: Vec<String> = hi
			.flags
			.iter()
			.map(|f| {
				if lo.flags.contains(f) {
					f.clone()
				} else {
					format!("High end: {f}")
				}
			})
			.collect();
		for f in &lo.flags {
			if !hi.flags.contains(f) {
				flags.push(format!("Low end: {f}"));
			}
		}

		let mut stop_flags = Vec::new();
		if a.tamanu_live == Some(true) {
			stop_flags.push(
				"Tamanu is already in use at the site: migrated records must not clash with records entered since go-live."
					.into(),
			);
		}
		if a.doc_contact == Some(DocLevel::None) && a.doc_dictionary == Some(DocLevel::None) {
			stop_flags.push(
				"Nobody at the client or vendor can explain the data, and there is no data dictionary."
					.into(),
			);
		}

		let has = |area: Area| areas.iter().any(|l| l.area == area);
		let e = &self.effort;
		let mut not_included = vec![
			format!(
				"Post go-live support ({} to {} data person-days per cutover).",
				fmt_num(e.post_go_live_days.low),
				fmt_num(e.post_go_live_days.high)
			),
			"Preparing data for Tamanu upgrades during the migration (not estimated).".into(),
			"FHIR integration on the deployment (a separate piece of work).".into(),
		];
		if has(Area::M10f) || has(Area::M10r) {
			not_included.push("Building Tamanu forms or registries that do not exist yet.".into());
		}
		if !has(Area::M10f) {
			not_included.push("Clinical forms: none counted.".into());
		}
		if !has(Area::M10r) {
			not_included.push("Disease or program forms: none counted.".into());
		}
		not_included.push("Finding and merging duplicate patient records.".into());
		not_included.push("Project management, infrastructure and development involvement.".into());

		// How much the client's documentation moves the estimate.
		let documentation_alternatives = (extras && given.doc == DocLevel::Partial).then(|| {
			let good = Some(DocLevel::Good);
			DocumentationAlternatives {
				good: self.alternative(
					&EstimateInput {
						doc_dictionary: good,
						doc_codes: good,
						doc_contact: good,
						doc_prior_exports: good,
						..a.clone()
					},
					ranged,
				),
				no_contact: self.alternative(
					&EstimateInput {
						doc_contact: Some(DocLevel::None),
						..a.clone()
					},
					ranged,
				),
			}
		});

		// What each unanswered question would do to the range once answered,
		// those that move it most first.
		let mut narrowing: Vec<(f64, f64, Narrowing)> = Vec::new();
		if ranged && extras {
			for q in questions().into_iter().filter(|q| (q.open)(self, a)) {
				let at = |end| {
					let mut b = a.clone();
					(q.fill)(self, &mut b, end);
					self.alternative(&b, true)
				};
				let (cheapest, costliest) = (at(End::Low), at(End::High));
				let swing = |s: fn(&Alternative) -> Span| {
					let (c, d) = (s(&cheapest), s(&costliest));
					(d.low - c.low) + (d.high - c.high)
				};
				let (months, days) = (swing(|x| x.months), swing(|x| x.person_days));
				if months > 0.0 || days > 0.0 {
					narrowing.push((
						months,
						days,
						Narrowing {
							question: q.label.into(),
							cheapest,
							costliest,
						},
					));
				}
			}
			narrowing.sort_by(|x, y| y.0.total_cmp(&x.0).then(y.1.total_cmp(&x.1)));
		}

		Estimate {
			model_version: self.version.clone(),
			tier: if ranged { a.tier } else { Some(given.tier) },
			confidence: given.confidence,
			documentation: given.doc,
			upper_bound: given.spreadsheet,
			areas,
			staffing,
			person_days: person_days.map(round_days),
			scoping_person_days: scoping.map(|s| s.map(round_days)),
			documentation_alternatives,
			narrowing: narrowing.into_iter().map(|(_, _, n)| n).collect(),
			workings,
			stop_flags,
			flags,
			unknown: given.unknown.clone(),
			not_included,
			narrows_after: "The range is wide because data quality and shape vary widely from one client to the next. A better estimate can be given after the initial analysis of the data (M0)."
				.into(),
		}
	}
}

/// Which end of a range an unanswered question is answered for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
	Low,
	High,
}

fn pick<T>(end: End, low: T, high: T) -> T {
	match end {
		End::Low => low,
		End::High => high,
	}
}

/// A question that can be left unanswered.
struct Question {
	/// The question as the form asks it.
	label: &'static str,
	/// Whether it is unanswered, and so priced at both ends.
	open: fn(&MigrationConfig, &EstimateInput) -> bool,
	/// Answer it with its cheapest answer for the low end, or its costliest
	/// for the high end.
	fill: fn(&MigrationConfig, &mut EstimateInput, End),
}

/// Every question that can be left unanswered, in the order they are answered
/// for an end: the tier first, since the counts asked follow the scope.
fn questions() -> Vec<Question> {
	vec![
		Question {
			label: "Closest to what they need",
			open: |_, a| a.tier.is_none(),
			fill: |_, a, end| match end {
				End::Low => a.tier = Some(Tier::ReferenceAndUsers),
				End::High => {
					a.tier = Some(Tier::FullEhr);
					// Every area, taking the costlier of each pair of alternatives.
					for area in Area::ALL {
						// Forms are counted, and a count left blank is none.
						if !matches!(
							area,
							Area::M4h | Area::M11f | Area::M12b | Area::M10f | Area::M10r
						) && !a.add.contains(&area)
							&& !a.remove.contains(&area)
						{
							a.add.push(area);
						}
					}
				}
			},
		},
		Question {
			label: "Total patients a day across all sites",
			open: |_, a| a.patients_per_day.is_none(),
			fill: |_, a, end| {
				a.patients_per_day = Some(pick(end, CaseBand::Small, CaseBand::Large));
			},
		},
		Question {
			label: "Years of visit history to move",
			// Asked when visits are, or could be, in scope.
			open: |c, a| {
				a.history_years.is_none()
					&& (a.tier.is_none()
						|| Area::ALL.into_iter().any(|x| {
							(matches!(x, Area::M4 | Area::M4h) || x.needs_encounters())
								&& c.picked(a, x)
						}))
			},
			fill: |_, a, end| a.history_years = Some(pick(end, CaseBand::Small, CaseBand::Large)),
		},
		Question {
			label: "Staff to be migrated",
			open: |_, a| a.many_users.is_none(),
			fill: |_, a, end| a.many_users = Some(end == End::High),
		},
		Question {
			label: "What they use now",
			open: |_, a| a.source_kind.is_none(),
			fill: |_, a, end| {
				a.source_kind = Some(pick(end, SourceKind::SingleSystem, SourceKind::Mix));
			},
		},
		Question {
			label: "Separate installs",
			open: |_, a| a.source_instances.is_none(),
			fill: |c, a, end| {
				a.source_instances = Some(pick(end, 1, c.source.installs.many_from));
			},
		},
		Question {
			label: "Database",
			open: |_, a| a.source_db.is_none() && a.source_kind != Some(SourceKind::Spreadsheets),
			fill: |_, a, end| {
				a.source_db = Some(pick(end, SourceDb::MysqlOrPostgres, SourceDb::Other));
			},
		},
		Question {
			label: "An up-to-date data dictionary",
			open: |_, a| a.doc_dictionary.is_none(),
			fill: |_, a, end| a.doc_dictionary = Some(pick(end, DocLevel::Good, DocLevel::None)),
		},
		Question {
			label: "Code lists can be exported",
			open: |_, a| a.doc_codes.is_none(),
			fill: |_, a, end| a.doc_codes = Some(pick(end, DocLevel::Good, DocLevel::None)),
		},
		Question {
			label: "Someone who knows how the data is stored",
			open: |_, a| a.doc_contact.is_none(),
			fill: |_, a, end| a.doc_contact = Some(pick(end, DocLevel::Good, DocLevel::None)),
		},
		Question {
			label: "Earlier exports, with their queries",
			// Asked only when the data dictionary is partly there or missing.
			open: |_, a| a.doc_prior_exports.is_none() && a.doc_dictionary != Some(DocLevel::Good),
			fill: |_, a, end| {
				a.doc_prior_exports = Some(pick(end, DocLevel::Good, DocLevel::None));
			},
		},
		Question {
			label: "Client reviews work",
			open: |_, a| a.review_speed.is_none(),
			fill: |_, a, end| {
				a.review_speed = Some(pick(end, ReviewSpeed::Fast, ReviewSpeed::Slow))
			},
		},
	]
}

/// A band's position in the size factors.
fn band_index(band: CaseBand) -> usize {
	match band {
		CaseBand::Small => 0,
		CaseBand::Medium => 1,
		CaseBand::Large => 2,
	}
}

/// A band as an estimate names it, such as "100 to 300 patients a day".
fn band_label(band: CaseBand, bands: CaseBands, what: &str) -> String {
	match band {
		CaseBand::Small => format!("under {} {what}", bands.medium_from),
		CaseBand::Medium => format!("{} to {} {what}", bands.medium_from, bands.large_over),
		CaseBand::Large => format!("over {} {what}", bands.large_over),
	}
}

/// "1 week" or "2 weeks".
fn weeks(n: f64) -> String {
	format!("{} week{}", fmt_num(n), if n == 1.0 { "" } else { "s" })
}

/// A range as prose: "2 weeks", "1 to 8 weeks", "6 person-days".
fn amount(span: Span, unit: &str) -> String {
	let n = |x: f64| fmt_num((x * 100.0).round() / 100.0);
	let plural = if span.high == 1.0 { "" } else { "s" };
	if span.low == span.high {
		format!("{} {unit}{plural}", n(span.high))
	} else {
		format!("{} to {} {unit}{plural}", n(span.low), n(span.high))
	}
}

fn capitalise(s: &str) -> String {
	let mut c = s.chars();
	c.next()
		.map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
}

/// A person-day total: the nearest five, or the nearest whole day below
/// thirty, where rounding to five distorts.
fn round_days(x: f64) -> f64 {
	if x < 30.0 {
		round_whole(x)
	} else {
		5.0 * round_whole(x / 5.0)
	}
}
