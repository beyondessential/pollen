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
	pub hours_per_month: f64,
	/// Size band to server to storage, in the order the calculator offers them.
	pub bands: Vec<ServerBox>,
	/// A facility server in BES cloud, whatever the deployment's size.
	pub facility: ServerBox,
	pub params: Params,
	pub display: Display,
	pub max_cloud_facilities: u32,
	pub wizard: WizardMapping,
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
}

/// How sure the model's owner is of a parameter's value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

/// What a parameter's value measures, so it can be shown in the right form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Unit {
	Usd,
	UsdPerGbMonth,
	/// A fraction, shown as a percentage.
	Share,
	Count,
	Multiple,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Param {
	pub label: String,
	pub unit: Unit,
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
	pub backups_db_pool_usd: Param,
	pub backups_db_deployments: Param,
	pub tupaia_prod_pool_usd: Param,
	pub tupaia_prod_share: Param,
	pub reporting_pipeline_pool_usd: Param,
	pub dagster_pool_usd: Param,
	pub replicas_sharing_pipeline: Param,
}

impl Params {
	/// Every parameter with its id, in authored order, for the assumptions panel.
	pub fn entries(&self) -> [(&'static str, &Param); 14] {
		[
			("gp3_usd_per_gb_month", &self.gp3_usd_per_gb_month),
			("snapshot_usd_per_gb_month", &self.snapshot_usd_per_gb_month),
			("base_per_box_usd", &self.base_per_box_usd),
			("clone_snapshot_multiple", &self.clone_snapshot_multiple),
			("clone_uptime", &self.clone_uptime),
			("windows_uplift_on_compute", &self.windows_uplift_on_compute),
			("platform_share_usd", &self.platform_share_usd),
			("backups_db_pool_usd", &self.backups_db_pool_usd),
			("backups_db_deployments", &self.backups_db_deployments),
			("tupaia_prod_pool_usd", &self.tupaia_prod_pool_usd),
			("tupaia_prod_share", &self.tupaia_prod_share),
			(
				"reporting_pipeline_pool_usd",
				&self.reporting_pipeline_pool_usd,
			),
			("dagster_pool_usd", &self.dagster_pool_usd),
			("replicas_sharing_pipeline", &self.replicas_sharing_pipeline),
		]
	}
}

/// Applied to displayed figures only; the breakdown is always cost price.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Display {
	/// Margin over cost, as a fraction (0.2 is 20%).
	pub margin: f64,
	pub currency: String,
	pub usd_to_currency: f64,
	pub round_monthly_to: f64,
	pub round_annual_to: f64,
}

/// How the priced wizard reads a plan's answers into model inputs.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WizardMapping {
	pub central_in_cloud: Condition,
	pub backups: Toggle,
	pub tupaia: Toggle,
	pub facility_count: Lookup,
	pub cloud_share: Lookup,
	pub notes: Vec<Note>,
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
		let mut seen = std::collections::BTreeSet::new();
		for b in &self.bands {
			if !seen.insert(b.name.as_str()) {
				return Err(AppError::custom(format!("band {} is listed twice", b.name)));
			}
		}
		if self.bands.is_empty() {
			return Err(AppError::custom("no size bands are priced"));
		}
		// Divisors must be positive, or a share becomes infinite.
		for (id, p) in [
			(
				"backups_db_deployments",
				&self.params.backups_db_deployments,
			),
			(
				"replicas_sharing_pipeline",
				&self.params.replicas_sharing_pipeline,
			),
		] {
			if p.value <= 0.0 {
				return Err(AppError::custom(format!("{id} must be positive")));
			}
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
