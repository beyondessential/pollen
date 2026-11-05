//! Read a plan's answers into cost model inputs for the priced wizard (spec
//! COST, The priced wizard). Which answers mean what is configuration, in the
//! `wizard` block of `pricing.ron`; this applies it.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::config::{Confidence, PricingConfig};
use super::model::{CloneMode, CostBreakdown, CostInput, LineId};
use crate::error::Result;
use crate::ruleset::{Answers, Ruleset, effective_answers, evaluate};

/// What the person pricing sets beside the plan. Never stored with it.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct PlanOverrides {
	#[serde(default)]
	pub cloud_facilities: Option<u32>,
	#[serde(default)]
	pub clone: Option<CloneMode>,
	#[serde(default)]
	pub clone_uptime: Option<f64>,
}

/// A cost input resting on an answer still open, priced one way for now.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OpenLine {
	pub id: LineId,
	pub label: String,
	/// Whether the line is in the figure as it stands.
	pub included: bool,
	/// What it adds each month, displayed as the lines of the breakdown are.
	pub display_monthly: f64,
}

/// The guessed cloud facility server count and what it rests on.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FacilityGuess {
	/// Facility servers in BES cloud as answered, before any setting.
	pub count: u32,
	/// All the deployment's facility servers, in BES cloud or not: the most a
	/// setting can put in BES cloud.
	pub total: u32,
	pub confidence: Confidence,
	pub basis: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlanCost {
	/// The derived size band, if the plan is sized.
	pub size: Option<String>,
	/// Why there is no figure, when there is none.
	pub missing: Option<String>,
	pub guess: FacilityGuess,
	/// The inputs priced, overrides applied.
	pub input: Option<CostInput>,
	pub cost: Option<CostBreakdown>,
	pub open: Vec<OpenLine>,
	pub notes: Vec<String>,
}

pub fn price_plan(
	config: &PricingConfig,
	ruleset: &Ruleset,
	answers: &Answers,
	overrides: &PlanOverrides,
) -> Result<PlanCost> {
	let w = &config.wizard;
	// The band for pricing leaves out the answers the model does not size by
	// (connected systems), whatever the ruleset's own size makes of them.
	let mut sizing = answers.clone();
	sizing.0.retain(|qid, _| !w.band_ignores.contains(qid));
	let evaluation = evaluate(ruleset, &sizing);
	let answers = effective_answers(ruleset, answers);

	let max = f64::from(config.max_cloud_facilities);
	let total = w.facility_count.pick(&answers).round().clamp(0.0, max);
	let guessed = (total * w.cloud_share.pick(&answers)).round();
	let guess = FacilityGuess {
		count: guessed as u32,
		total: total as u32,
		confidence: w.facility_count.confidence,
		basis: format!("{} {}", w.facility_count.basis, w.cloud_share.basis),
	};
	let notes = w
		.notes
		.iter()
		.filter(|n| n.when.eval(&answers))
		.map(|n| n.text.clone())
		.collect();

	let size = evaluation.derived.get("size").cloned();
	let missing = match &size {
		None => Some("Answer the first two questions to see a cost.".to_string()),
		Some(s) if config.band(s).is_none() => Some(format!("The {s} band is not priced yet.")),
		Some(_) => None,
	};
	let Some(band) = size.clone().filter(|_| missing.is_none()) else {
		return Ok(PlanCost {
			size,
			missing,
			guess,
			input: None,
			cost: None,
			open: Vec::new(),
			notes,
		});
	};

	let input = CostInput {
		band,
		central_in_cloud: w.central_in_cloud.eval(&answers),
		cloud_facilities: overrides
			.cloud_facilities
			.map_or(guess.count, |n| n.min(guess.total)),
		clone: overrides.clone.unwrap_or_default(),
		clone_uptime: overrides.clone_uptime,
		backups: w.backups.on.eval(&answers),
		tupaia: w.tupaia.on.eval(&answers),
		windows_central: false,
		windows_facilities: 0,
		windows_clone: false,
	};
	let cost = config.cost(&input)?;

	let db_gb = config.band(&input.band).map_or(0.0, |b| b.db_gb);
	let mut open = Vec::new();
	if w.backups.open.eval(&answers) {
		open.push(OpenLine {
			id: LineId::BackupsOpsShare,
			label: "Backups and clone feeding".into(),
			included: input.backups,
			display_monthly: config.display_amount(config.backups_share(db_gb)),
		});
	}
	if w.tupaia.open.eval(&answers) {
		open.push(OpenLine {
			id: LineId::TupaiaReporting,
			label: "Tupaia reporting".into(),
			included: input.tupaia,
			display_monthly: config.display_amount(config.tupaia_addon(db_gb)),
		});
	}

	Ok(PlanCost {
		size,
		missing: None,
		guess,
		input: Some(input),
		cost: Some(cost),
		open,
		notes,
	})
}
