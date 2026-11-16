//! The internal data migration effort model and its configuration (spec MIG).
//! Separate from the hosting cost model: its own configuration, its own
//! address, and no monetary figure.

pub mod bes_led;
pub mod client_prepared;
pub mod config;

pub use bes_led::{
	AreaLine, ConfidenceLevel, DocLevel, Estimate, EstimateInput, Narrowing, Staffing, StaffingRow,
};
pub use client_prepared::{CaseBand, ImportEstimate, ImportInput, ImportProject};
pub use config::{
	Adjust, Area, AreaRow, BUNDLED_DATA_MIGRATION, Basis, CaseBands, Combine, MigrationConfig,
	Span, Tier, TierRow, valid_access_path,
};

/// Round half up to a whole number, the way the reference model does.
pub(crate) fn round_whole(x: f64) -> f64 {
	(x + 0.5).floor()
}

/// Round to the nearest half.
pub(crate) fn round_half(x: f64) -> f64 {
	round_whole(x * 2.0) / 2.0
}

/// A whole number with thousands separators, as the bands are offered.
pub(crate) fn thousands(n: u32) -> String {
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

/// A number as prose carries it: no trailing ".0".
pub(crate) fn fmt_num(x: f64) -> String {
	if x.fract() == 0.0 {
		format!("{x:.0}")
	} else {
		format!("{x}")
	}
}
