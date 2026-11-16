//! The migration configuration: every range, multiplier, share and threshold
//! the effort model uses (spec MIG, The migration configuration). Authored in
//! RON at `data-migration.ron` and bundled at build time.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{AppError, Result};

/// The migration configuration bundled into the binary.
pub const BUNDLED_DATA_MIGRATION: &str = include_str!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/../../data-migration.ron"
));

/// One area of a migration. Declared in the order the areas are worked, which
/// is the order an estimate lists them in.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
pub enum Area {
	M0,
	M1,
	M2,
	M3,
	M4,
	M4h,
	M5,
	M6,
	M7,
	M8,
	M9,
	M10f,
	M10r,
	M11,
	M11f,
	M12b,
	M12f,
	M13,
	M14,
}

impl Area {
	pub const ALL: [Area; 19] = [
		Area::M0,
		Area::M1,
		Area::M2,
		Area::M3,
		Area::M4,
		Area::M4h,
		Area::M5,
		Area::M6,
		Area::M7,
		Area::M8,
		Area::M9,
		Area::M10f,
		Area::M10r,
		Area::M11,
		Area::M11f,
		Area::M12b,
		Area::M12f,
		Area::M13,
		Area::M14,
	];

	/// Worked after encounters, and shared between the data people.
	pub fn after_encounters(self) -> bool {
		self >= Area::M5
	}

	/// Its records sit on an encounter in Tamanu. Registries, appointments and
	/// documents are patient-level and do not.
	pub fn needs_encounters(self) -> bool {
		matches!(
			self,
			Area::M5
				| Area::M6
				| Area::M7
				| Area::M8
				| Area::M9
				| Area::M10f
				| Area::M12b
				| Area::M12f
				| Area::M14
		)
	}

	/// Sized as a count times the range for one, and splittable between people.
	pub fn counted(self) -> bool {
		matches!(self, Area::M10f | Area::M10r)
	}

	pub fn code(self) -> String {
		format!("{self:?}")
	}
}

/// The starting scope of a BES-led migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Tier {
	#[serde(alias = "ReferenceAndUsers")]
	ReferenceAndUsers,
	#[serde(alias = "PatientRegister")]
	PatientRegister,
	#[serde(alias = "PatientHistory")]
	PatientHistory,
	#[serde(alias = "FullEhr")]
	FullEhr,
}

impl Tier {
	pub const ALL: [Tier; 4] = [
		Tier::ReferenceAndUsers,
		Tier::PatientRegister,
		Tier::PatientHistory,
		Tier::FullEhr,
	];
}

/// What an area's starting range rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Basis {
	/// Taken from a previous BES-led migration.
	Calibrated,
	/// Set from the team's experience.
	TeamExperience,
	/// Worked out from another area's range.
	Derived,
}

/// A low and a high end.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Span {
	pub low: f64,
	pub high: f64,
}

impl Span {
	pub const ZERO: Span = Span {
		low: 0.0,
		high: 0.0,
	};

	pub fn map(self, f: impl Fn(f64) -> f64) -> Span {
		Span {
			low: f(self.low),
			high: f(self.high),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AreaRow {
	pub area: Area,
	/// The area as an estimate names it.
	pub name: String,
	/// How the area is offered as a choice. An area without one comes with a
	/// tier or another answer.
	#[serde(default)]
	pub pick: Option<String>,
	/// Areas sharing a group are alternatives: at most one is picked.
	#[serde(default)]
	pub group: Option<String>,
	/// Elapsed weeks, for one of it.
	pub weeks: Span,
	pub basis: Basis,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasedSpan {
	pub weeks: Span,
	pub basis: Basis,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TierRow {
	pub tier: Tier,
	pub label: String,
	pub hint: String,
	pub areas: Vec<Area>,
}

/// What one answer adds: elapsed weeks to the timeline and data person-days,
/// each a range (equal ends for a single figure).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adjust {
	pub weeks: Span,
	pub days: Span,
}

impl Adjust {
	/// This adjustment `n` times over.
	pub fn times(self, n: f64) -> Adjust {
		Adjust {
			weeks: self.weeks.map(|x| x * n),
			days: self.days.map(|x| x * n),
		}
	}
}

/// What a documentation answer of Partly or No multiplies every area by; Yes
/// multiplies by one.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocFactor {
	pub partial: f64,
	pub none: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Documentation {
	/// Someone who knows how the data is stored.
	pub contact: DocFactor,
	/// Code lists can be exported.
	pub codes: DocFactor,
	/// An up-to-date data dictionary.
	pub dictionary: DocFactor,
	/// No earlier exports with their queries, asked only when the data
	/// dictionary is partly there or missing.
	pub no_prior_exports: f64,
}

/// Separate installs of the source system, in three bands: one, from
/// `some_from`, and from `many_from`.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installs {
	pub some_from: u32,
	pub many_from: u32,
	pub some: Adjust,
	pub many: Adjust,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
	/// A database other than MySQL or Postgres.
	pub other_db: Adjust,
	/// Several systems, or a mix including spreadsheets.
	pub several: Adjust,
	pub installs: Installs,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
	/// The bands of total patients a day across all sites.
	pub patients: CaseBands,
	/// The bands of years of visit history to migrate.
	pub years: CaseBands,
	/// The multiplier for each band of patients a day: under, within and over
	/// the middle band.
	pub patients_factor: [f64; 3],
	/// The multiplier for each band of years of history, applied after the
	/// patients a day.
	pub years_factor: [f64; 3],
}

/// Staff to be migrated, past staff included when records are attributed to
/// them: M2 takes `weeks` up to `many_over` staff and `many_weeks` over it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Users {
	pub weeks: f64,
	pub many_weeks: f64,
	pub many_over: u32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sites {
	/// The first site's cutover.
	pub cutover_weeks: f64,
	pub cutover_days: Span,
	/// Each site beyond the first.
	pub extra: Adjust,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timeline {
	pub default_people: u32,
	pub weeks_per_month: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effort {
	pub days_per_week: f64,
	pub m0_share: f64,
	pub area_share: f64,
	/// Added to the data total for review.
	pub review_allowance: f64,
	/// Added to the person-days for each data person beyond the first,
	/// compounding, for keeping the work aligned.
	pub alignment_per_extra_person: f64,
	pub post_go_live_days: Span,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidenceRule {
	pub low_from_unknowns: usize,
	pub high_max_unknowns: usize,
}

/// The bands a program's rough case numbers are given in: under
/// `medium_from`, from `medium_from` to `large_over`, and over `large_over`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CaseBands {
	pub medium_from: u32,
	pub large_over: u32,
}

/// Days each of a program's forms takes, by the program's case band.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseDays {
	pub small: Span,
	pub medium: Span,
	pub large: Span,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientPrepared {
	pub templates_days: f64,
	pub registry_days: f64,
	pub extra_site_days: Span,
	pub contingency_days: Span,
	pub buffer_days: f64,
	pub case_bands: CaseBands,
	pub case_days: CaseDays,
	pub days_per_elapsed_week: f64,
	pub max_projects: usize,
	/// What an unanswered number of sites is read as at the high end.
	pub unanswered_sites: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationConfig {
	pub model: String,
	/// The model's version as shown beside an estimate.
	pub version: String,
	/// The path segment the estimator lives under.
	pub access_path: String,
	pub areas: Vec<AreaRow>,
	pub m0_full_ehr: BasedSpan,
	pub tiers: Vec<TierRow>,
	/// The tier an estimate starts from when none is chosen.
	pub unsure_tier: Tier,
	pub source: Source,
	pub size: Size,
	pub users: Users,
	pub sites: Sites,
	/// Already in use at the site: checking for conflicts.
	pub tamanu_live: Adjust,
	/// Added to the high end of the timeline when the client is slow to review.
	pub slow_review_weeks: f64,
	pub timeline: Timeline,
	pub effort: Effort,
	pub confidence: ConfidenceRule,
	pub client_prepared: ClientPrepared,
	pub documentation: Documentation,
	pub combine: Combine,
	/// The most any count answer is read as.
	pub max_count: u32,
}

/// How ranges are added up into the timeline and the person-days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Combine {
	/// Every low added up and every high added up.
	Stacked,
	/// Each range spread around its midpoint, the spreads combined as the root
	/// of the sum of their squares.
	Independent,
}

/// Whether a path segment can serve as an internal edition's address: it
/// becomes a URL segment, so it has to survive as one.
pub fn valid_access_path(path: &str) -> bool {
	path.len() >= 8
		&& path
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

impl MigrationConfig {
	pub fn from_ron(source: &str) -> Result<Self> {
		let config: Self = ron::from_str(source).map_err(AppError::custom)?;
		config.validate()?;
		Ok(config)
	}

	/// The row for an area. Validation guarantees every area has one.
	pub fn area(&self, area: Area) -> &AreaRow {
		self.areas
			.iter()
			.find(|r| r.area == area)
			.expect("every area is configured")
	}

	/// The row for a tier. Validation guarantees every tier has one.
	pub fn tier(&self, tier: Tier) -> &TierRow {
		self.tiers
			.iter()
			.find(|r| r.tier == tier)
			.expect("every tier is configured")
	}

	/// Reject a path segment shared with the hosting cost edition, which would
	/// put two editions at one address.
	pub fn check_distinct_from(&self, other_access_path: &str) -> Result<()> {
		if self.access_path == other_access_path {
			return Err(AppError::custom(
				"the migration access_path must differ from the pricing access_path",
			));
		}
		Ok(())
	}

	fn validate(&self) -> Result<()> {
		if !valid_access_path(&self.access_path) {
			return Err(AppError::custom(
				"access_path must be at least 8 characters of letters, digits, '-' or '_'",
			));
		}
		for area in Area::ALL {
			match self.areas.iter().filter(|r| r.area == area).count() {
				1 => {}
				0 => return Err(AppError::custom(format!("area {area:?} has no range"))),
				_ => return Err(AppError::custom(format!("area {area:?} is listed twice"))),
			}
		}
		for tier in Tier::ALL {
			match self.tiers.iter().filter(|r| r.tier == tier).count() {
				1 => {}
				0 => return Err(AppError::custom(format!("tier {tier:?} is not configured"))),
				_ => return Err(AppError::custom(format!("tier {tier:?} is listed twice"))),
			}
		}
		let c = &self.client_prepared;
		let spans = self
			.areas
			.iter()
			.map(|r| (r.name.as_str(), r.weeks))
			.chain([
				("m0_full_ehr", self.m0_full_ehr.weeks),
				("sites.cutover_days", self.sites.cutover_days),
				("effort.post_go_live_days", self.effort.post_go_live_days),
				("client_prepared.extra_site_days", c.extra_site_days),
				("client_prepared.case_days.small", c.case_days.small),
				("client_prepared.case_days.medium", c.case_days.medium),
				("client_prepared.case_days.large", c.case_days.large),
				("client_prepared.contingency_days", c.contingency_days),
			])
			.chain(
				self.adjustments()
					.into_iter()
					.flat_map(|(what, a)| [(what, a.weeks), (what, a.days)]),
			)
			.collect::<Vec<_>>();
		for (what, span) in spans {
			if !(span.low >= 0.0 && span.high >= span.low) {
				return Err(AppError::custom(format!(
					"{what}: a range needs 0 <= low <= high"
				)));
			}
		}
		// Divisors must be positive, or a figure becomes infinite.
		if self.timeline.weeks_per_month <= 0.0 || c.days_per_elapsed_week <= 0.0 {
			return Err(AppError::custom(
				"weeks_per_month and days_per_elapsed_week must be positive",
			));
		}
		if !(1..=3).contains(&self.timeline.default_people) {
			return Err(AppError::custom("default_people must be 1, 2 or 3"));
		}
		if self.size.patients.medium_from >= self.size.patients.large_over
			|| self.size.years.medium_from >= self.size.years.large_over
		{
			return Err(AppError::custom(
				"size.patients and size.years need medium_from below large_over",
			));
		}
		let d = c.case_days;
		let ascending = |a: Span, b: Span| a.low <= b.low && a.high <= b.high;
		if !(ascending(d.small, d.medium) && ascending(d.medium, d.large)) {
			return Err(AppError::custom(
				"client_prepared.case_days needs small <= medium <= large at each end",
			));
		}
		if !(c.registry_days >= 0.0 && c.templates_days >= 0.0) {
			return Err(AppError::custom(
				"client_prepared.registry_days and templates_days must not be negative",
			));
		}
		if c.case_bands.medium_from >= c.case_bands.large_over {
			return Err(AppError::custom(
				"client_prepared.case_bands.medium_from must be below large_over",
			));
		}
		if self.max_count == 0 || c.max_projects == 0 {
			return Err(AppError::custom(
				"max_count and max_projects must be positive",
			));
		}
		if c.unanswered_sites == 0 || c.unanswered_sites > self.max_count {
			return Err(AppError::custom(
				"client_prepared.unanswered_sites must be at least 1 and at most max_count",
			));
		}
		let i = &self.source.installs;
		if !(2 <= i.some_from && i.some_from < i.many_from && i.many_from <= self.max_count) {
			return Err(AppError::custom(
				"source.installs needs 2 <= some_from < many_from <= max_count",
			));
		}
		let d = &self.documentation;
		let doc_factors = [
			d.contact.partial,
			d.contact.none,
			d.codes.partial,
			d.codes.none,
			d.dictionary.partial,
			d.dictionary.none,
			d.no_prior_exports,
		];
		let factors = self
			.size
			.patients_factor
			.iter()
			.chain(&self.size.years_factor)
			.chain(&doc_factors);
		let fixed = [
			self.users.weeks,
			self.users.many_weeks,
			self.sites.cutover_weeks,
			self.slow_review_weeks,
			self.effort.days_per_week,
			self.effort.m0_share,
			self.effort.area_share,
			self.effort.review_allowance,
			self.effort.alignment_per_extra_person,
		];
		if factors.clone().any(|f| !f.is_finite() || *f <= 0.0)
			|| fixed.iter().any(|f| !f.is_finite() || *f < 0.0)
		{
			return Err(AppError::custom(
				"size and documentation factors must be positive, and the users, sites, review and effort figures not negative",
			));
		}
		Ok(())
	}

	/// Every adjustment an answer can add, named for validation.
	fn adjustments(&self) -> Vec<(&'static str, Adjust)> {
		let s = &self.source;
		vec![
			("source.other_db", s.other_db),
			("source.several", s.several),
			("source.installs.some", s.installs.some),
			("source.installs.many", s.installs.many),
			("sites.extra", self.sites.extra),
			("tamanu_live", self.tamanu_live),
		]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bundled_configuration_is_well_formed() {
		let config = MigrationConfig::from_ron(BUNDLED_DATA_MIGRATION).expect("bundled parses");
		let pricing = crate::pricing::PricingConfig::from_ron(crate::pricing::BUNDLED_PRICING)
			.expect("bundled pricing parses");
		config
			.check_distinct_from(&pricing.access_path)
			.expect("the two editions live at different paths");
	}

	#[test]
	fn a_missing_or_repeated_area_is_rejected() {
		let missing = BUNDLED_DATA_MIGRATION.replacen("area: M14,", "area: M13,", 1);
		assert!(MigrationConfig::from_ron(&missing).is_err());
	}

	#[test]
	fn a_shared_path_is_rejected() {
		let config = MigrationConfig::from_ron(BUNDLED_DATA_MIGRATION).unwrap();
		assert!(config.check_distinct_from(&config.access_path).is_err());
	}
}
