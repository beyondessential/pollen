//! The hosting cost model: a deployment's monthly cost to BES as a sum of
//! lines (spec COST, The cost model). Pure arithmetic over the pricing
//! configuration, pinned to the cent by the reference fixtures.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::config::{PricingConfig, ServerBox};
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
	BackupsDbShare,
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
}

/// The displayed figures: margin and currency applied, then rounded.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DisplayTotal {
	pub currency: String,
	pub margin: f64,
	pub monthly: f64,
	pub annual: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CostBreakdown {
	pub lines: Vec<CostLine>,
	/// The unrounded sum of the lines.
	pub monthly_usd: f64,
	pub annual_usd: f64,
	pub display: DisplayTotal,
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

	pub fn backups_share(&self) -> f64 {
		self.params.backups_db_pool_usd.value / self.params.backups_db_deployments.value
	}

	pub fn tupaia_addon(&self) -> f64 {
		let p = &self.params;
		p.tupaia_prod_share.value * p.tupaia_prod_pool_usd.value
			+ (p.reporting_pipeline_pool_usd.value + p.dagster_pool_usd.value)
				/ p.replicas_sharing_pipeline.value
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
		if !(0.0..=1.0).contains(&uptime) {
			return Err(AppError::BadRequest(
				"clone uptime must be between 0 and 1".into(),
			));
		}
		let p = &self.params;
		let pct = |f: f64| format!("{}%", (f * 100.0).round());
		let mut lines = Vec::new();

		if input.central_in_cloud {
			lines.push(CostLine {
				id: LineId::CentralBox,
				label: "Central server".into(),
				detail: Some(format!("{}, {} GB", central.instance, central.volume_gb)),
				monthly_usd: self.box_cost(central),
			});
		}
		let n = input.cloud_facilities;
		lines.push(CostLine {
			id: LineId::CloudFacilities,
			label: "Cloud facility servers".into(),
			detail: (n > 0).then(|| {
				format!(
					"{n} × {}, {} GB",
					self.facility.instance, self.facility.volume_gb
				)
			}),
			monthly_usd: f64::from(n) * self.box_cost(&self.facility),
		});
		if input.clone != CloneMode::Off {
			lines.push(CostLine {
				id: LineId::Clone,
				label: "Clone".into(),
				detail: Some(match input.clone {
					CloneMode::AlwaysOn => "Always on".into(),
					_ => format!("Running {} of the month", pct(uptime)),
				}),
				monthly_usd: self.clone_cost(central, uptime),
			});
		}
		lines.push(CostLine {
			id: LineId::PlatformShare,
			label: "Platform share".into(),
			detail: None,
			monthly_usd: p.platform_share_usd.value,
		});
		if input.backups {
			lines.push(CostLine {
				id: LineId::BackupsDbShare,
				label: "Backups share".into(),
				detail: None,
				monthly_usd: self.backups_share(),
			});
		}
		if input.tupaia {
			lines.push(CostLine {
				id: LineId::TupaiaReporting,
				label: "Tupaia reporting".into(),
				detail: None,
				monthly_usd: self.tupaia_addon(),
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
			});
		}

		let monthly_usd: f64 = lines.iter().map(|l| l.monthly_usd).sum();
		let annual_usd = monthly_usd * 12.0;
		let d = &self.display;
		let scale = (1.0 + d.margin) * d.usd_to_currency;
		let round = |v: f64, step: f64| {
			if step > 0.0 {
				(v / step).round() * step
			} else {
				v
			}
		};
		Ok(CostBreakdown {
			lines,
			monthly_usd,
			annual_usd,
			display: DisplayTotal {
				currency: d.currency.clone(),
				margin: d.margin,
				monthly: round(monthly_usd * scale, d.round_monthly_to),
				annual: round(annual_usd * scale, d.round_annual_to),
			},
		})
	}
}
