//! The internal hosting cost model and its configuration (spec COST).

pub mod config;
pub mod estimate;
pub mod model;
pub mod wizard;

pub use config::{BUNDLED_PRICING, Confidence, Display, Param, PricingConfig, ServerBox};
pub use estimate::{EstimatorView, SizeArgs, SizeEstimate};
pub use model::{CloneMode, CostBreakdown, CostInput, CostLine, DisplayTotal, LineId};
pub use wizard::{FacilityGuess, OpenLine, PlanCost, PlanOverrides, price_plan};
