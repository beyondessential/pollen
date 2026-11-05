//! The hosting cost model: a deployment's monthly cost to BES as a sum of
//! lines (spec COST, The cost model). Pure arithmetic over the pricing
//! configuration, pinned to the cent by the reference fixtures.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::config::{Confidence, Param, PricingConfig, ServerBox};
use crate::error::{AppError, Result};

/// The clone used to test upgrades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CloneMode {
	Off,
	/// Stopped by default and run for a share of the month.
	#[default]
	Standard,
	AlwaysOn,
}

/// What a deployment is priced from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CostInput {
	/// A size band named in the pricing configuration.
	pub band: String,
	/// Whether the central server is in BES cloud. A client-hosted central
	/// server costs BES nothing, though its clone still runs in BES cloud.
	pub central_in_cloud: bool,
	/// Facility servers in BES cloud.
	pub cloud_facilities: u32,
	pub clone: CloneMode,
	/// How much of the month a standard clone runs, as a fraction. Omitted
	/// takes the configured share. Ignored unless the clone is standard.
	#[serde(default)]
	pub clone_uptime: Option<f64>,
	pub backups: bool,
	pub tupaia: bool,
	/// A Windows central server. Applies only while the central is in BES cloud.
	#[serde(default)]
	pub windows_central: bool,
	/// How many of the cloud facility servers run Windows (capped at their count).
	#[serde(default)]
	pub windows_facilities: u32,
	#[serde(default)]
	pub windows_clone: bool,
}

/// A line of the breakdown. Ids are stable, so tests and the UI can key on them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LineId {
	CentralBox,
	CloudFacilities,
	Clone,
	PlatformShare,
	BackupsOpsShare,
	TupaiaReporting,
	WindowsExtra,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CostLine {
	pub id: LineId,
	pub label: String,
	/// What the line is made of, such as the server and its storage.
	pub detail: Option<String>,
	/// Unrounded monthly cost to BES.
	pub monthly_usd: f64,
	/// The weakest confidence among the parameters the line rests on, so a
	/// figure made of estimates is not mistaken for a fact.
	pub confidence: Confidence,
}

/// The displayed figures: margin and currency applied, then each line rounded
/// to the configured step. The monthly figure is the sum of the rounded lines,
/// and the annual figure twelve times it, so what is shown adds up.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisplayTotal {
	pub currency: String,
	pub margin: f64,
	/// Each line's displayed figure, in the order of the lines.
	pub lines: Vec<f64>,
	pub monthly: f64,
	pub annual: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CostBreakdown {
	/// The version of the model these figures come from.
	pub model_version: String,
	pub lines: Vec<CostLine>,
	/// The unrounded sum of the lines.
	pub monthly_usd: f64,
	pub annual_usd: f64,
	pub display: DisplayTotal,
}

/// Storage as shown, with a thousands separator.
fn gb(v: f64) -> String {
	let n = v.round() as u64;
	if n >= 1000 {
		format!("{},{:03}", n / 1000, n % 1000)
	} else {
		n.to_string()
	}
}

/// The weakest confidence among the parameters a line rests on.
fn weakest(params: &[&Param]) -> Confidence {
	params
		.iter()
		.map(|p| p.confidence)
		.max()
		.unwrap_or(Confidence::Decided)
}

impl PricingConfig {
	fn compute(&self, b: &ServerBox) -> f64 {
		b.usd_per_hour * self.hours_per_month
	}

	/// Storage and its snapshots at a given snapshot multiple, plus the base.
	fn fixed(&self, b: &ServerBox, snapshot_multiple: f64) -> f64 {
		let p = &self.params;
		p.base_per_box_usd.value
			+ b.volume_gb * p.gp3_usd_per_gb_month.value
			+ b.volume_gb * snapshot_multiple * p.snapshot_usd_per_gb_month.value
	}

	/// A production server running all month.
	pub fn box_cost(&self, b: &ServerBox) -> f64 {
		self.compute(b) + self.fixed(b, b.snapshot_multiple)
	}

	/// A clone pair (central and one facility): storage always, compute for
	/// the share of the month it runs.
	fn clone_cost(&self, central: &ServerBox, uptime: f64) -> f64 {
		let snap = self.params.clone_snapshot_multiple.value;
		self.fixed(central, snap)
			+ self.fixed(&self.facility, snap)
			+ uptime * (self.compute(central) + self.compute(&self.facility))
	}

	/// The reporting part of pgro over the DB size of the deployments that have
	/// a replica, per GB per month.
	pub fn pgro_reporting_usd_per_gb(&self) -> f64 {
		let p = &self.params;
		p.pgro_pool_usd.value * (1.0 - p.pgro_ops_share.value) / p.pgro_replica_gb_total.value
	}

	/// The backups part of pgro over the DB size of every deployment with
	/// backups on, per GB per month.
	pub fn pgro_backups_usd_per_gb(&self) -> f64 {
		let p = &self.params;
		p.pgro_pool_usd.value * p.pgro_ops_share.value / p.fleet_db_gb_total.value
	}

	/// The parts of the Tupaia add-on that do not scale with DB size, split
	/// evenly across the deployments with a replica.
	pub fn tupaia_flat(&self) -> f64 {
		let p = &self.params;
		(p.tamanu_share_of_tupaia_compute.value * p.tupaia_prod_compute_usd.value
			+ p.dagster_pool_usd.value)
			/ p.pgro_replicas.value
	}

	/// What reporting adds over and above backups: the flat share plus the
	/// reporting part of pgro by DB size.
	pub fn tupaia_addon(&self, db_gb: f64) -> f64 {
		self.tupaia_flat() + self.pgro_reporting_usd_per_gb() * db_gb
	}

	/// Taking backups and feeding clones: the Tamanu-ops part of pgro by DB size.
	pub fn backups_share(&self, db_gb: f64) -> f64 {
		self.pgro_backups_usd_per_gb() * db_gb
	}

	/// A cost as displayed: margin and currency applied, then rounded to the
	/// configured step, so every figure on a page is shown the same way.
	pub fn display_amount(&self, usd: f64) -> f64 {
		let d = &self.display;
		let v = usd * (1.0 + d.margin) * d.usd_to_currency;
		if d.round_monthly_to > 0.0 {
			(v / d.round_monthly_to).round() * d.round_monthly_to
		} else {
			v
		}
	}

	/// Price a deployment.
	pub fn cost(&self, input: &CostInput) -> Result<CostBreakdown> {
		let central = self.band(&input.band).ok_or_else(|| {
			AppError::BadRequest(format!("the {} band is not priced", input.band))
		})?;
		let uptime = match input.clone {
			CloneMode::Off => 0.0,
			CloneMode::AlwaysOn => 1.0,
			CloneMode::Standard => input.clone_uptime.unwrap_or(self.params.clone_uptime.value),
		};
		// A negative zero would print as "-0%".
		let uptime = if uptime == 0.0 { 0.0 } else { uptime };
		if !(0.0..=1.0).contains(&uptime) {
			return Err(AppError::BadRequest(
				"clone uptime must be between 0 and 1".into(),
			));
		}
		if input.cloud_facilities > self.max_cloud_facilities {
			return Err(AppError::BadRequest(format!(
				"at most {} facility servers can be priced",
				self.max_cloud_facilities
			)));
		}
		let p = &self.params;
		let pct = |f: f64| format!("{}%", (f * 100.0).round());
		// Boxes rest on rates read from price lists and bills.
		let rates = weakest(&[
			&p.gp3_usd_per_gb_month,
			&p.snapshot_usd_per_gb_month,
			&p.base_per_box_usd,
		]);
		let mut lines = Vec::new();

		if input.central_in_cloud {
			lines.push(CostLine {
				id: LineId::CentralBox,
				label: "Central server".into(),
				detail: Some(format!(
					"{}, {}\u{a0}GB",
					central.instance,
					gb(central.volume_gb)
				)),
				monthly_usd: self.box_cost(central),
				confidence: rates,
			});
		}
		let n = input.cloud_facilities;
		lines.push(CostLine {
			id: LineId::CloudFacilities,
			label: "Cloud facility servers".into(),
			detail: (n > 0).then(|| {
				format!(
					"{n} × {}, {}\u{a0}GB",
					self.facility.instance,
					gb(self.facility.volume_gb)
				)
			}),
			monthly_usd: f64::from(n) * self.box_cost(&self.facility),
			confidence: rates,
		});
		if input.clone != CloneMode::Off {
			lines.push(CostLine {
				id: LineId::Clone,
				label: "Smoke testing clone".into(),
				detail: Some(match input.clone {
					CloneMode::AlwaysOn => "Always on".into(),
					_ => format!("Compute {} of the month, storage always", pct(uptime)),
				}),
				monthly_usd: self.clone_cost(central, uptime),
				confidence: weakest(&[&p.clone_snapshot_multiple, &p.clone_uptime]).max(rates),
			});
		}
		lines.push(CostLine {
			id: LineId::PlatformShare,
			label: "Platform share".into(),
			detail: Some("BES infrastructure".into()),
			monthly_usd: p.platform_share_usd.value,
			confidence: p.platform_share_usd.confidence,
		});
		let db_gb = central.db_gb;
		// A share that rounds away to nothing is not a line.
		if input.backups && self.backups_share(db_gb) > 0.0 {
			lines.push(CostLine {
				id: LineId::BackupsOpsShare,
				label: "Backups and clone feeding".into(),
				detail: Some("Share of the data cluster, by database size".into()),
				monthly_usd: self.backups_share(db_gb),
				confidence: weakest(&[&p.pgro_pool_usd, &p.pgro_ops_share, &p.fleet_db_gb_total])
					.max(self.db_size_by_band.confidence),
			});
		}
		if input.tupaia {
			lines.push(CostLine {
				id: LineId::TupaiaReporting,
				label: "Tupaia reporting".into(),
				detail: Some("Over and above backups".into()),
				monthly_usd: self.tupaia_addon(db_gb),
				confidence: weakest(&[
					&p.tupaia_prod_compute_usd,
					&p.tamanu_share_of_tupaia_compute,
					&p.pgro_pool_usd,
					&p.pgro_ops_share,
					&p.pgro_replicas,
					&p.pgro_replica_gb_total,
					&p.dagster_pool_usd,
				])
				.max(self.db_size_by_band.confidence),
			});
		}

		// Windows adds to compute only, on each Windows server BES runs.
		let uplift = p.windows_uplift_on_compute.value;
		let windows_central = input.windows_central && input.central_in_cloud;
		let windows_facilities = input.windows_facilities.min(n);
		let windows_clone = input.windows_clone && input.clone != CloneMode::Off;
		let mut windows = 0.0;
		if windows_central {
			windows += uplift * self.compute(central);
		}
		windows += uplift * self.compute(&self.facility) * f64::from(windows_facilities);
		if windows_clone {
			windows += uplift * uptime * (self.compute(central) + self.compute(&self.facility));
		}
		if windows > 0.0 {
			let mut parts = Vec::new();
			if windows_central {
				parts.push("central".to_string());
			}
			if windows_facilities > 0 {
				parts.push(format!("{windows_facilities} facility"));
			}
			if windows_clone {
				parts.push("clone".to_string());
			}
			lines.push(CostLine {
				id: LineId::WindowsExtra,
				label: "Windows".into(),
				detail: Some(format!("{} on compute: {}", pct(uplift), parts.join(", "))),
				monthly_usd: windows,
				confidence: p.windows_uplift_on_compute.confidence.max(rates),
			});
		}

		let monthly_usd: f64 = lines.iter().map(|l| l.monthly_usd).sum();
		let annual_usd = monthly_usd * 12.0;
		let d = &self.display;
		let display_lines: Vec<f64> = lines
			.iter()
			.map(|l| self.display_amount(l.monthly_usd))
			.collect();
		let display_monthly: f64 = display_lines.iter().sum();
		Ok(CostBreakdown {
			model_version: self.version.clone(),
			lines,
			monthly_usd,
			annual_usd,
			display: DisplayTotal {
				currency: d.currency.clone(),
				margin: d.margin,
				lines: display_lines,
				monthly: display_monthly,
				annual: display_monthly * 12.0,
			},
		})
	}
}
