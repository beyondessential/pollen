//! Evaluate a ruleset against a user's answers: derive values, collect the
//! union of triggered consequences, surface active guidance and the set of
//! currently-visible questions, and reduce to a verdict (spec WIZ, Rule engine
//! model).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::answers::Answers;
use super::model::{Consequence, DerivationKind, Ruleset, Severity};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Evaluation {
	/// Derived values keyed by derivation id (e.g. `size` → `Medium`).
	pub derived: BTreeMap<String, String>,
	/// The ids of questions currently shown, in ruleset order (spec WIZ,
	/// visibility). The frontend renders exactly these.
	pub visible_questions: Vec<String>,
	/// Every triggered consequence, in ruleset order.
	pub consequences: Vec<TriggeredConsequence>,
	/// Guidance whose condition currently holds.
	pub guidance: Vec<TriggeredGuidance>,
	pub verdict: Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TriggeredConsequence {
	pub id: String,
	pub source: String,
	pub consequence: Consequence,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TriggeredGuidance {
	pub at: String,
	pub message: String,
}

/// The viability verdict: the worst severity present across triggered
/// consequences. Default-severity consequences don't move it off `Clear`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Verdict {
	Clear,
	NonDefault,
	Blocking,
}

pub fn evaluate(ruleset: &Ruleset, answers: &Answers) -> Evaluation {
	let derived = derive(ruleset, answers);

	let visible_questions = ruleset
		.questions
		.iter()
		.filter(|q| q.visible_if.eval(answers))
		.map(|q| q.id.clone())
		.collect();

	let consequences: Vec<TriggeredConsequence> = ruleset
		.rules
		.iter()
		.filter(|r| r.when.eval(answers))
		.map(|r| TriggeredConsequence {
			id: r.id.clone(),
			source: r.source.clone(),
			consequence: r.consequence.clone(),
		})
		.collect();

	let guidance: Vec<TriggeredGuidance> = ruleset
		.guidance
		.iter()
		.filter(|g| g.when.eval(answers))
		.map(|g| TriggeredGuidance {
			at: g.at.clone(),
			message: g.message.clone(),
		})
		.collect();

	let verdict = if consequences
		.iter()
		.any(|c| c.consequence.severity == Severity::Blocking)
	{
		Verdict::Blocking
	} else if consequences
		.iter()
		.any(|c| c.consequence.severity == Severity::NonDefault)
	{
		Verdict::NonDefault
	} else {
		Verdict::Clear
	};

	Evaluation {
		derived,
		visible_questions,
		consequences,
		guidance,
		verdict,
	}
}

fn derive(ruleset: &Ruleset, answers: &Answers) -> BTreeMap<String, String> {
	let mut out = BTreeMap::new();
	for d in &ruleset.derivations {
		match &d.kind {
			DerivationKind::HighestBand { questions, labels } => {
				let mut max: Option<usize> = None;
				for qid in questions {
					if let (Some(q), Some(answer)) = (ruleset.question(qid), answers.one(qid))
						&& let Some(ix) = q.option_index(answer)
					{
						max = Some(max.map_or(ix, |m| m.max(ix)));
					}
				}
				if let Some(label) = max.and_then(|ix| labels.get(ix)) {
					out.insert(d.id.clone(), label.clone());
				}
			}
		}
	}
	out
}
