//! The ruleset data model. A ruleset is authored in RON and deserializes into
//! these types; the engine ([`super::engine`]) evaluates them against a user's
//! answers. See spec WIZ (Rule engine model).

use serde::{Deserialize, Serialize};

use super::condition::Condition;

/// A complete ruleset: the questions asked, values derived from answers, the
/// rules that fire requirements and consequences, and forward guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ruleset {
	pub questions: Vec<Question>,
	#[serde(default)]
	pub derivations: Vec<Derivation>,
	pub rules: Vec<Rule>,
	#[serde(default)]
	pub guidance: Vec<Guidance>,
}

impl Ruleset {
	/// Parse a ruleset from its RON authoring form.
	pub fn from_ron(source: &str) -> crate::error::Result<Self> {
		ron::from_str(source).map_err(crate::error::AppError::custom)
	}

	pub fn question(&self, id: &str) -> Option<&Question> {
		self.questions.iter().find(|q| q.id == id)
	}

	/// Check the stable-id discipline: question, option (within a question), and
	/// rule ids are unique. Run on load (spec WIZ, stable-id migration).
	pub fn validate(&self) -> crate::error::Result<()> {
		use crate::error::AppError;
		use std::collections::HashSet;

		let mut question_ids = HashSet::new();
		for q in &self.questions {
			if !question_ids.insert(q.id.as_str()) {
				return Err(AppError::custom(format!("duplicate question id: {}", q.id)));
			}
			let mut option_ids = HashSet::new();
			for o in &q.options {
				if !option_ids.insert(o.id.as_str()) {
					return Err(AppError::custom(format!(
						"duplicate option id {} in question {}",
						o.id, q.id
					)));
				}
			}
		}

		let mut rule_ids = HashSet::new();
		for r in &self.rules {
			if !rule_ids.insert(r.id.as_str()) {
				return Err(AppError::custom(format!("duplicate rule id: {}", r.id)));
			}
		}

		Ok(())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
	/// Permanent identifier. Never reused or repurposed (spec WIZ, stable-id).
	pub id: String,
	pub kind: QuestionKind,
	pub label: String,
	#[serde(default)]
	pub help: Option<String>,
	#[serde(default)]
	pub options: Vec<Opt>,
	/// Shown only when this holds; otherwise hidden (spec WIZ, visibility).
	#[serde(default = "Condition::always")]
	pub visible_if: Condition,
}

impl Question {
	/// Ordinal of an option within this question (its order in `options`),
	/// used by band derivations.
	pub fn option_index(&self, option_id: &str) -> Option<usize> {
		self.options.iter().position(|o| o.id == option_id)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum QuestionKind {
	/// Pick one option.
	Single,
	/// Pick any number of options.
	Multi,
	/// Pick one option from an ordered set of bands (low to high).
	Band,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Opt {
	pub id: String,
	pub label: String,
	#[serde(default)]
	pub note: Option<String>,
	/// In a multi-select, choosing this option clears the others, and choosing
	/// any other clears this one (e.g. a "none of these" choice).
	#[serde(default)]
	pub exclusive: bool,
}

/// A value derived from answers, surfaced in the artifact (e.g. the size band).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Derivation {
	pub id: String,
	pub kind: DerivationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DerivationKind {
	/// The highest band reached across the named band questions, mapped to a
	/// label by ordinal (`labels[max ordinal]`). Absent if none are answered.
	/// When `bump_when` holds, the band is raised one step (capped at the top).
	HighestBand {
		questions: Vec<String>,
		labels: Vec<String>,
		#[serde(default)]
		bump_when: Option<Condition>,
	},
}

/// A rule: when its condition holds, its consequence is added to the artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
	pub id: String,
	/// Topic this rule belongs to, for by-topic grouping in the artifact.
	pub source: String,
	pub when: Condition,
	pub consequence: Consequence,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Consequence {
	#[serde(default)]
	pub severity: Severity,
	#[serde(default)]
	pub types: Vec<ConsequenceType>,
	pub status: Status,
	pub audience: Audience,
	pub title: String,
	pub detail: String,
	#[serde(default)]
	pub cost: Option<Cost>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Cost {
	pub tier: String,
	#[serde(default)]
	pub ballpark: Option<String>,
}

/// The viability axis (spec WIZ, Severity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub enum Severity {
	#[default]
	Default,
	NonDefault,
	Blocking,
}

/// The "this is worse" axis (spec WIZ, Consequence type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum ConsequenceType {
	Cost,
	Operational,
	Capability,
	Support,
}

/// The technical-versus-contractual line (spec WIZ, Status).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum Status {
	Requirement,
	Advisory,
	Referral,
}

/// Which reader a consequence is grouped under in the by-audience view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum Audience {
	Client,
	Bes,
	Record,
}

/// Forward guidance shown at a question before a constraint is reached
/// (spec WIZ, Visibility and forward guidance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guidance {
	/// The question id at which to surface this guidance.
	pub at: String,
	pub when: Condition,
	pub message: String,
}
