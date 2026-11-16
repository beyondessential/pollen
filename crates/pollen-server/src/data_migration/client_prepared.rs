//! The client-prepared route of the effort model: the client formats the data
//! into BES templates, and BES validates and imports it. Sized in data team
//! days for each program (spec MIG, Client-prepared imports).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::config::{MigrationConfig, Span};
use super::{fmt_num, round_half};

/// Roughly how many cases a program has in total, as one of the configured
/// bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CaseBand {
	/// Fewer than `case_bands.medium_from`.
	Small,
	/// From `case_bands.medium_from` to `case_bands.large_over`.
	Medium,
	/// More than `case_bands.large_over`.
	Large,
}

/// A whole number with thousands separators, as the bands are offered.
fn thousands(n: u32) -> String {
	let digits = n.to_string();
	let mut out = String::new();
	for (i, d) in digits.chars().enumerate() {
		if i > 0 && (digits.len() - i).is_multiple_of(3) {
			out.push(',');
		}
		out.push(d);
	}
	out
}

/// One program of a client-prepared engagement.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct ImportProject {
	#[serde(default)]
	pub sites: Option<u32>,
	/// Whether the program has a registry, which brings in its patients.
	#[serde(default)]
	pub registry: Option<bool>,
	/// Forms the program's data is imported into.
	#[serde(default)]
	pub forms: u32,
	/// Roughly how many cases the program has in total.
	#[serde(default)]
	pub cases: Option<CaseBand>,
}

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct ImportInput {
	/// Whether BES import templates already exist for these registries and forms.
	#[serde(default)]
	pub templates_exist: Option<bool>,
	#[serde(default)]
	pub projects: Vec<ImportProject>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ImportProjectLine {
	/// The project's position in the engagement, from 1.
	pub number: u32,
	/// Sites going live; none when not answered.
	pub sites: Option<u32>,
	/// Whether it has a registry; none when not answered.
	pub registry: Option<bool>,
	pub forms: u32,
	/// Data team days, to the nearest half.
	pub days: Span,
	/// Elapsed weeks once the formatted data arrives, to the nearest half.
	pub weeks: Span,
}

/// A client-prepared estimate, as displayed.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ImportEstimate {
	pub model_version: String,
	pub projects: Vec<ImportProjectLine>,
	/// Days to build the import templates, when they are not known to exist:
	/// from none when that is not answered.
	pub templates_days: Option<Span>,
	/// Data team days for the engagement, to the nearest half.
	pub total_days: Span,
	/// How each project's days are put together.
	pub workings: Vec<String>,
	/// Each assumption made.
	pub flags: Vec<String>,
	pub not_included: Vec<String>,
}

impl MigrationConfig {
	/// Estimate a client-prepared import from its projects.
	pub fn estimate_import(&self, a: &ImportInput) -> ImportEstimate {
		let c = &self.client_prepared;
		let mut flags = Vec::new();
		let mut total = Span::ZERO;

		// Unanswered questions are priced at both ends: the cheapest answer
		// for the low end, the costliest for the high end.
		let templates = match a.templates_exist {
			Some(true) => None,
			Some(false) => Some(Span {
				low: c.templates_days,
				high: c.templates_days,
			}),
			None => {
				flags.push(format!(
					"Unknown whether BES import templates exist for these registries and forms: 0 to {} day(s).",
					fmt_num(c.templates_days)
				));
				Some(Span {
					low: 0.0,
					high: c.templates_days,
				})
			}
		};
		if a.projects.len() > c.max_projects {
			flags.push(format!(
				"Only the first {} programs are costed: {} more listed were left out.",
				c.max_projects,
				a.projects.len() - c.max_projects
			));
		}
		if a.projects.is_empty() {
			flags.push("No programs listed: only the import templates are costed.".into());
		}

		let mut projects = Vec::new();
		for (i, p) in a.projects.iter().take(c.max_projects).enumerate() {
			let number = i as u32 + 1;
			let name = format!("Program {number}");
			// The registry counts once, whatever the number of sites.
			let registry_days = match p.registry {
				Some(true) => Span {
					low: c.registry_days,
					high: c.registry_days,
				},
				Some(false) => Span::ZERO,
				None => {
					flags.push(format!(
						"{name}: unknown whether it has a registry: priced from none to one."
					));
					Span {
						low: 0.0,
						high: c.registry_days,
					}
				}
			};
			let forms = p.forms.min(self.max_count);
			let sites = match p.sites.map(|n| n.min(self.max_count)) {
				Some(n) if n > 0 => Some(n),
				_ => {
					flags.push(format!(
						"{name}: number of sites unknown: priced from 1 to {}.",
						c.unanswered_sites
					));
					None
				}
			};
			let extra_sites = match sites {
				Some(n) => Span {
					low: f64::from(n - 1),
					high: f64::from(n - 1),
				},
				None => Span {
					low: 0.0,
					high: f64::from(c.unanswered_sites - 1),
				},
			};
			// Each form costs a range set by the program's case numbers.
			let per_form = match p.cases {
				Some(CaseBand::Small) => c.case_days.small,
				Some(CaseBand::Medium) => c.case_days.medium,
				Some(CaseBand::Large) => c.case_days.large,
				None => {
					flags.push(format!(
						"{name}: case numbers unknown: priced from under {} to over {}.",
						thousands(c.case_bands.medium_from),
						thousands(c.case_bands.large_over)
					));
					Span {
						low: c.case_days.small.low,
						high: c.case_days.large.high,
					}
				}
			};
			let n = f64::from(forms);
			let days = Span {
				low: n * per_form.low
					+ extra_sites.low * c.extra_site_days.low
					+ c.contingency_days.low
					+ registry_days.low
					+ c.buffer_days,
				high: n * per_form.high
					+ extra_sites.high * c.extra_site_days.high
					+ c.contingency_days.high
					+ registry_days.high
					+ c.buffer_days,
			};
			// The total adds up the programs as shown, to the nearest half day.
			total.low += round_half(days.low);
			total.high += round_half(days.high);

			if p.registry == Some(false) && forms == 0 {
				flags.push(format!("{name}: no registry or forms listed."));
			}
			projects.push(ImportProjectLine {
				number,
				sites,
				registry: p.registry,
				forms,
				days: days.map(round_half),
				weeks: days.map(|d| round_half(d / c.days_per_elapsed_week)),
			});
		}
		if let Some(t) = templates {
			total.low += t.low;
			total.high += t.high;
		}

		let range = |s: Span| format!("{} to {}", fmt_num(s.low), fmt_num(s.high));
		let workings = vec![
			format!(
				"Each form, by the program's case numbers: {} days under {} cases, {} for {} to {}, {} over {}.",
				range(c.case_days.small),
				thousands(c.case_bands.medium_from),
				range(c.case_days.medium),
				thousands(c.case_bands.medium_from),
				thousands(c.case_bands.large_over),
				range(c.case_days.large),
				thousands(c.case_bands.large_over)
			),
			format!(
				"The registry: {} day, once per program.",
				fmt_num(c.registry_days)
			),
			format!(
				"Each site beyond the first: {} days, for variations in how it was implemented.",
				range(c.extra_site_days)
			),
			format!(
				"Contingency: {} days per program.",
				range(c.contingency_days)
			),
			format!(
				"Standing buffer: {} days per program.",
				fmt_num(c.buffer_days)
			),
			format!(
				"Import templates: {} day(s), once per engagement, if they do not exist.",
				fmt_num(c.templates_days)
			),
			format!(
				"Timeline: a program takes its data team days divided by {} in weeks, once the client's formatted data arrives. Programs can run in parallel across the data team.",
				fmt_num(c.days_per_elapsed_week)
			),
		];

		ImportEstimate {
			model_version: self.version.clone(),
			projects,
			templates_days: templates,
			total_days: total.map(round_half),
			workings,
			flags,
			not_included: vec![
				"The client's time formatting the data.".into(),
				"Building or changing Tamanu forms and registries.".into(),
				"Reports and DHIS2 set-up.".into(),
				"Post go-live support.".into(),
				"Removing and re-importing data if the client's data changes after import.".into(),
			],
		}
	}
}
