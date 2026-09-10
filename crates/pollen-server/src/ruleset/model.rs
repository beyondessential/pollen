//! The ruleset data model. A ruleset is authored in RON and deserializes into
//! these types; the engine ([`super::engine`]) evaluates them against a user's
//! answers. See spec WIZ (Rule engine model).

use serde::{Deserialize, Serialize};

use super::condition::Condition;

/// A complete ruleset: the questions asked, values derived from answers, the
/// rules that fire requirements and consequences, and forward guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ruleset {
	/// The flow's sections, in presentation order. Questions name one via
	/// `Question::section` (spec WIZ, Question flow).
	#[serde(default)]
	pub sections: Vec<Section>,
	pub questions: Vec<Question>,
	#[serde(default)]
	pub derivations: Vec<Derivation>,
	pub rules: Vec<Rule>,
	#[serde(default)]
	pub guidance: Vec<Guidance>,
	/// Compute requirement profiles, surfaced in the artifact for each server or
	/// device class present in the deployment (spec WIZ, Compute requirements).
	#[serde(default)]
	pub requirements: Vec<Requirement>,
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

		let mut requirement_ids = HashSet::new();
		for r in &self.requirements {
			if !requirement_ids.insert(r.id.as_str()) {
				return Err(AppError::custom(format!(
					"duplicate requirement id: {}",
					r.id
				)));
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
	/// Which section of the flow this question sits in. Unset means the first.
	#[serde(default)]
	pub section: Option<String>,
	/// The option id assumed when the question is visible but left unanswered
	/// (spec WIZ, assumed defaults). A question with no default and no `unsure`
	/// option must be answered.
	#[serde(default)]
	pub default: Option<String>,
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
	/// This option means "I don't know". Choosing it assumes nothing and
	/// records the question as an open item (spec WIZ, open items).
	#[serde(default)]
	pub unsure: bool,
	/// Shown inline when this option is selected, so the cost of leaving the
	/// blessed path lands at the point of choice.
	#[serde(default)]
	pub warn: Option<String>,
}

/// A named group of questions, presented together.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Section {
	pub id: String,
	pub label: String,
	/// Collapsed on arrival, for sections a non-technical user can skip.
	#[serde(default)]
	pub collapsed: bool,
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

/// A compute requirement profile for one class of server or device. Surfaced in
/// the artifact when its `when` condition holds, i.e. when that class is present
/// in the deployment (spec WIZ, Compute requirements). The figures are the
/// recommended base-level specs; scaling for larger deployments is advised
/// separately by BES.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
	/// Permanent identifier. Never reused or repurposed (spec WIZ, stable-id).
	pub id: String,
	/// Surfaced only when this holds (e.g. the class is present in the mix).
	pub when: Condition,
	/// The server or device class this profile describes, e.g. "Central server".
	pub class: String,
	/// A short line on who provisions this class and when it appears.
	#[serde(default)]
	pub summary: Option<String>,
	/// The spec rows (processor, memory, storage, network, and so on).
	pub specs: Vec<Spec>,
	/// An optional caveat shown beneath the rows.
	#[serde(default)]
	pub note: Option<String>,
}

/// One row of a compute requirement: a labelled figure such as
/// `("Memory", "16 GB")`.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Spec {
	pub label: String,
	pub value: String,
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
	/// The BES pricing and partnerships team: anything that moves what the
	/// deployment costs or what BES can commit to supporting.
	Pricing,
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
