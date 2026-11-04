//! The calculator's size estimator: work out the size band, and the number of
//! facility servers, from the wizard's sizing questions (spec COST, The
//! calculator). The band comes from the ruleset's own derivation, never a
//! second implementation of it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::config::PricingConfig;
use crate::ruleset::{Answer, Answers, Question, QuestionKind, Ruleset, evaluate};

/// A sizing question as the estimator offers it.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EstimatorQuestion {
	pub id: String,
	pub label: String,
	pub options: Vec<EstimatorOption>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EstimatorOption {
	pub id: String,
	pub label: String,
}

/// The questions the estimator asks, taken from the ruleset.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EstimatorView {
	pub sizing: Vec<EstimatorQuestion>,
	/// The yes/no about connected systems.
	pub integrations: Option<EstimatorToggle>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EstimatorToggle {
	pub label: String,
	pub hint: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SizeArgs {
	/// Option ids keyed by sizing question id. Questions left out are unanswered.
	#[serde(default)]
	pub answers: BTreeMap<String, String>,
	/// Whether any other system connects.
	#[serde(default)]
	pub integrations: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SizeEstimate {
	/// The derived band; none until a sizing question is answered.
	pub band: Option<String>,
	/// Facility servers in BES cloud if they all run there, once the facilities
	/// question is answered.
	pub cloud_facilities: Option<u32>,
}

impl PricingConfig {
	fn clamp_facilities(&self, v: f64) -> u32 {
		v.round().clamp(0.0, f64::from(self.max_cloud_facilities)) as u32
	}

	/// The options of a question that mean something, without "I'm not sure".
	fn real_options(q: &Question) -> impl Iterator<Item = &crate::ruleset::Opt> {
		q.options.iter().filter(|o| !o.unsure)
	}

	pub fn estimator_view(&self, ruleset: &Ruleset) -> EstimatorView {
		let sizing = self
			.estimator
			.sizing
			.iter()
			.filter_map(|id| ruleset.question(id))
			.map(|q| EstimatorQuestion {
				id: q.id.clone(),
				label: q.label.clone(),
				// Clearing a choice is how a question goes back to "not sure".
				options: Self::real_options(q)
					.map(|o| EstimatorOption {
						id: o.id.clone(),
						label: o.label.clone(),
					})
					.collect(),
			})
			.collect();
		EstimatorView {
			sizing,
			integrations: ruleset
				.question(&self.estimator.integrations)
				.map(|q| EstimatorToggle {
					label: q.label.clone(),
					hint: self.estimator.integrations_hint.clone(),
				}),
		}
	}

	/// Derive the band, and the facility server count, from sizing answers.
	pub fn estimate_size(&self, ruleset: &Ruleset, args: &SizeArgs) -> SizeEstimate {
		let mut answers = Answers::default();
		for (qid, oid) in &args.answers {
			let known = self.estimator.sizing.contains(qid)
				&& ruleset
					.question(qid)
					.is_some_and(|q| q.option_index(oid).is_some());
			if known {
				answers.set(qid, Answer::One(oid.clone()));
			}
		}
		if args.integrations
			&& let Some(q) = ruleset.question(&self.estimator.integrations)
			&& q.kind == QuestionKind::Multi
			&& let Some(o) = Self::real_options(q).find(|o| !o.exclusive)
		{
			answers.set(&q.id, Answer::Many(vec![o.id.clone()]));
		}
		let band = evaluate(ruleset, &answers).derived.get("size").cloned();
		let cloud_facilities = answers
			.answered(&self.estimator.facilities)
			.then(|| self.clamp_facilities(self.wizard.facility_count.pick(&answers)));
		SizeEstimate {
			band,
			cloud_facilities,
		}
	}

	/// For each band, the facility servers if they all run in BES cloud: the
	/// facilities option at the band's own position, since the sizing questions
	/// share the bands' order.
	pub fn band_facility_guesses(&self, ruleset: &Ruleset) -> Vec<u32> {
		let options: Vec<&str> = ruleset
			.question(&self.estimator.facilities)
			.map(|q| Self::real_options(q).map(|o| o.id.as_str()).collect())
			.unwrap_or_default();
		(0..self.bands.len())
			.map(|i| {
				let Some(oid) = options.get(i).or(options.last()) else {
					return 0;
				};
				let mut answers = Answers::default();
				answers.set(&self.estimator.facilities, Answer::One((*oid).to_string()));
				self.clamp_facilities(self.wizard.facility_count.pick(&answers))
			})
			.collect()
	}
}
