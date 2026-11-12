//! The pricing configuration: every rate, share and server size the cost model
//! uses, each parameter with its confidence and basis (spec COST, The pricing
//! configuration). Authored in RON at `pricing.ron` and bundled at build time.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{AppError, Result};
use crate::ruleset::Condition;

/// The pricing configuration bundled into the binary.
pub const BUNDLED_PRICING: &str =
	include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../pricing.ron"));

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PricingConfig {
	pub model: String,
	/// The model's version as shown beside a figure.
	pub version: String,
	/// The path segment the internal edition lives under.
	pub access_path: String,
	pub hours_per_month: f64,
	/// Size band to server to storage, in the order the calculator offers them.
	pub bands: Vec<ServerBox>,
	/// How sure the bands' estimated database sizes are.
	pub db_size_by_band: Tagged,
	/// A facility server in BES cloud, whatever the deployment's size.
	pub facility: ServerBox,
	pub params: Params,
	pub display: Display,
	pub max_cloud_facilities: u32,
	pub estimator: Estimator,
	pub wizard: WizardMapping,
}

/// The calculator's size estimator: which ruleset questions it asks.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Estimator {
	/// The sizing questions, in the order they are asked.
	pub sizing: Vec<String>,
	/// The sizing question whose answer also gives the facility server count.
	pub facilities: String,
}

/// One server: its instance, hourly rate, storage, and how many copies of that
/// storage its snapshots hold.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ServerBox {
	pub name: String,
	pub instance: String,
	pub usd_per_hour: f64,
	pub volume_gb: f64,
	pub snapshot_multiple: f64,
	/// The database size estimated from the band (not an input). Only bands have one.
	#[serde(default)]
	pub db_gb: f64,
}

/// A confidence and its basis, for something that is not a single number.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tagged {
	pub confidence: Confidence,
	pub basis: String,
}

/// How sure the model's owner is of a parameter's value.
/// Ordered from most to least certain, so the weakest of several is the maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
pub enum Confidence {
	/// Settled by the model's owner.
	Decided,
	/// Read from a published rate or an observed bill.
	Read,
	/// An allocation of shared costs, expected to move.
	Estimated,
	/// A working assumption awaiting real figures.
	Assumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Param {
	pub value: f64,
	pub confidence: Confidence,
	pub basis: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Params {
	pub gp3_usd_per_gb_month: Param,
	pub snapshot_usd_per_gb_month: Param,
	pub base_per_box_usd: Param,
	pub clone_snapshot_multiple: Param,
	pub clone_uptime: Param,
	pub windows_uplift_on_compute: Param,
	pub platform_share_usd: Param,
	pub tupaia_prod_compute_usd: Param,
	pub tamanu_share_of_tupaia_compute: Param,
	pub pgro_pool_usd: Param,
	pub pgro_replicas: Param,
	pub pgro_replica_gb_total: Param,
	pub fleet_db_gb_total: Param,
	pub s3_backup_pool_usd: Param,
	pub demo_cluster_pool_usd: Param,
	pub demo_environments: Param,
	pub pgro_ops_share: Param,
	pub dagster_pool_usd: Param,
}

/// Applied to displayed figures only; the breakdown is always cost price.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Display {
	/// Margin over cost, as a fraction (0.2 is 20%).
	pub margin: f64,
	pub currency: String,
	pub usd_to_currency: f64,
	/// Each displayed line rounds to a multiple of this.
	pub round_monthly_to: f64,
}

/// How the priced wizard reads a plan's answers into model inputs.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WizardMapping {
	/// Questions whose answers do not count towards the size band for pricing.
	pub band_ignores: Vec<String>,
	pub central_in_cloud: Condition,
	pub backups: Toggle,
	pub tupaia: Toggle,
	pub facility_count: Lookup,
	pub cloud_share: CloudShare,
	/// The question the cloud facility slider takes the place of.
	pub slider_in_place_of: String,
	pub notes: Vec<Note>,
}

/// What share of the facility servers are in BES cloud, read from where they run.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudShare {
	pub all: Condition,
	pub mix: Condition,
	pub mix_share: f64,
	pub confidence: Confidence,
	pub basis: String,
}

impl CloudShare {
	pub fn pick(&self, answers: &crate::ruleset::Answers) -> f64 {
		if self.all.eval(answers) {
			1.0
		} else if self.mix.eval(answers) {
			self.mix_share
		} else {
			0.0
		}
	}
}

/// A yes/no input read from the answers, and whether the answer behind it is
/// still open.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toggle {
	pub on: Condition,
	pub open: Condition,
}

/// A value picked by the first matching row; no match is zero.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lookup {
	pub value: Vec<LookupRow>,
	pub confidence: Confidence,
	pub basis: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LookupRow {
	pub when: Condition,
	pub value: f64,
}

impl Lookup {
	pub fn pick(&self, answers: &crate::ruleset::Answers) -> f64 {
		self.value
			.iter()
			.find(|row| row.when.eval(answers))
			.map_or(0.0, |row| row.value)
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
	pub when: Condition,
	pub text: String,
}

impl PricingConfig {
	pub fn from_ron(source: &str) -> Result<Self> {
		let config: Self = ron::from_str(source).map_err(AppError::custom)?;
		config.validate()?;
		Ok(config)
	}

	/// The server for a size band, matched by the name the ruleset derives.
	pub fn band(&self, name: &str) -> Option<&ServerBox> {
		self.bands.iter().find(|b| b.name == name)
	}

	fn validate(&self) -> Result<()> {
		// It becomes a URL segment, so it has to survive as one.
		let path = &self.access_path;
		if path.len() < 8
			|| !path
				.chars()
				.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
		{
			return Err(AppError::custom(
				"access_path must be at least 8 characters of letters, digits, '-' or '_'",
			));
		}
		let mut seen = std::collections::BTreeSet::new();
		for b in &self.bands {
			if !seen.insert(b.name.as_str()) {
				return Err(AppError::custom(format!("band {} is listed twice", b.name)));
			}
		}
		if self.bands.is_empty() {
			return Err(AppError::custom("no size bands are priced"));
		}
		// Divisors must be positive, or a rate becomes infinite.
		for (id, p) in [
			("pgro_replicas", &self.params.pgro_replicas),
			("pgro_replica_gb_total", &self.params.pgro_replica_gb_total),
			("fleet_db_gb_total", &self.params.fleet_db_gb_total),
			("demo_environments", &self.params.demo_environments),
		] {
			if p.value <= 0.0 {
				return Err(AppError::custom(format!("{id} must be positive")));
			}
		}
		if !(0.0..=1.0).contains(&self.params.pgro_ops_share.value) {
			return Err(AppError::custom("pgro_ops_share must be between 0 and 1"));
		}
		let d = &self.display;
		// Shown figures are whole units, so a step has to be one for the lines
		// to visibly add up to the headline.
		if d.round_monthly_to < 1.0 || d.round_monthly_to.fract() != 0.0 {
			return Err(AppError::custom("round_monthly_to must be a whole number"));
		}
		if d.usd_to_currency <= 0.0 || d.margin <= -1.0 {
			return Err(AppError::custom(
				"margin and exchange rate must be positive",
			));
		}
		let mix = self.wizard.cloud_share.mix_share;
		if !(0.0..=1.0).contains(&mix) {
			return Err(AppError::custom("mix_share must be between 0 and 1"));
		}
		let uptime = self.params.clone_uptime.value;
		if !(0.0..=1.0).contains(&uptime) {
			return Err(AppError::custom("clone_uptime must be between 0 and 1"));
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bundled_pricing_is_well_formed() {
		let config = PricingConfig::from_ron(BUNDLED_PRICING).expect("bundled pricing parses");
		// Every band the ruleset derives must be priced, or the wizard would
		// report a sized plan as unpriced.
		let ruleset = crate::ruleset::Ruleset::from_ron(crate::ruleset::BUNDLED_RULESET).unwrap();
		for d in &ruleset.derivations {
			let crate::ruleset::DerivationKind::HighestBand { labels, .. } = &d.kind;
			for label in labels {
				assert!(
					config.band(label).is_some(),
					"size band {label} is not priced"
				);
			}
		}
	}
}
