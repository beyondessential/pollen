//! Evaluate a ruleset against a user's answers: derive values, collect the
//! union of triggered consequences, surface active guidance and the set of
//! currently-visible questions, and reduce to a verdict (spec WIZ, Rule engine
//! model).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::answers::{Answer, Answers};
use super::model::{Consequence, DerivationKind, QuestionKind, Ruleset, Severity, Spec};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Evaluation {
	/// Derived values keyed by derivation id (e.g. `size` → `Medium`).
	pub derived: BTreeMap<String, String>,
	/// The ids of questions currently shown, in ruleset order (spec WIZ,
	/// visibility). The frontend renders exactly these.
	pub visible_questions: Vec<String>,
	/// Every triggered consequence, in ruleset order.
	pub consequences: Vec<TriggeredConsequence>,
	/// The compute requirements for the classes present in the deployment, in
	/// ruleset order (spec WIZ, Compute requirements).
	pub requirements: Vec<TriggeredRequirement>,
	/// Guidance whose condition currently holds.
	pub guidance: Vec<TriggeredGuidance>,
	/// Visible questions left unanswered whose blessed-path default the engine
	/// applied on the user's behalf (spec WIZ, assumed defaults).
	pub assumed: Vec<Assumed>,
	/// Visible questions the user marked unsure, or left blank where an unsure
	/// option was available. These make the artifact interim.
	pub open_items: Vec<String>,
	/// Visible questions that must be answered: no default to fall back on and
	/// no unsure option to decline with. These block finalising.
	pub required: Vec<String>,
	pub verdict: Verdict,
}

/// A default the engine applied because the question was left unanswered.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Assumed {
	pub question: String,
	pub option: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TriggeredConsequence {
	pub id: String,
	pub source: String,
	pub consequence: Consequence,
}

/// A compute requirement whose class is present in the deployment. Carries the
/// profile's content (the `when` condition that selected it is not on the wire).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TriggeredRequirement {
	pub id: String,
	pub class: String,
	pub summary: Option<String>,
	pub specs: Vec<Spec>,
	pub note: Option<String>,
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
	let (answers, assumed) = apply_defaults(ruleset, answers);
	let answers = &answers;
	let derived = derive(ruleset, answers);

	let visible_questions: Vec<String> = ruleset
		.questions
		.iter()
		.filter(|q| q.visible_if.eval(answers))
		.map(|q| q.id.clone())
		.collect();

	// Split the still-unresolved visible questions. A question the user marked
	// unsure, or left blank where declining was offered, is an open item the
	// artifact carries forward; one with no way to decline must be answered.
	let mut open_items = Vec::new();
	let mut required = Vec::new();
	for qid in &visible_questions {
		let Some(q) = ruleset.question(qid) else {
			continue;
		};
		let has_unsure = q.options.iter().any(|o| o.unsure);
		let answered_unsure =
			q.options.iter().filter(|o| o.unsure).any(|o| {
				answers.one(qid) == Some(o.id.as_str()) || answers.many(qid).contains(&o.id)
			});
		if answered_unsure {
			open_items.push(qid.clone());
		} else if !answers.answered(qid) {
			if has_unsure {
				open_items.push(qid.clone());
			} else {
				required.push(qid.clone());
			}
		}
	}

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

	// Requirements are gated on the same defaulted answers as consequences, so an
	// assumed hosting choice surfaces the classes it implies.
	let requirements: Vec<TriggeredRequirement> = ruleset
		.requirements
		.iter()
		.filter(|r| r.when.eval(answers))
		.map(|r| TriggeredRequirement {
			id: r.id.clone(),
			class: r.class.clone(),
			summary: r.summary.clone(),
			specs: r.specs.clone(),
			note: r.note.clone(),
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
		requirements,
		guidance,
		assumed,
		open_items,
		required,
		verdict,
	}
}

/// Fill in the blessed-path answer for every visible question left blank that
/// declares one (spec WIZ, assumed defaults).
///
/// Applying a default can reveal a question that has a default of its own, so
/// this runs to a fixed point rather than in a single pass. The question count
/// bounds the loop: each round either settles at least one more question or
/// stops.
fn apply_defaults(ruleset: &Ruleset, answers: &Answers) -> (Answers, Vec<Assumed>) {
	let mut effective = answers.clone();
	let mut assumed = Vec::new();
	for _ in 0..=ruleset.questions.len() {
		let mut changed = false;
		for q in &ruleset.questions {
			let Some(default) = &q.default else { continue };
			if !q.visible_if.eval(&effective) || effective.answered(&q.id) {
				continue;
			}
			let answer = match q.kind {
				QuestionKind::Multi => Answer::Many(vec![default.clone()]),
				QuestionKind::Single | QuestionKind::Band => Answer::One(default.clone()),
			};
			effective.set(&q.id, answer);
			assumed.push(Assumed {
				question: q.id.clone(),
				option: default.clone(),
			});
			changed = true;
		}
		if !changed {
			break;
		}
	}
	// A default applied on one round can be hidden again by a later one, so
	// only report assumptions whose question is still visible.
	assumed.retain(|a| {
		ruleset
			.question(&a.question)
			.is_some_and(|q| q.visible_if.eval(&effective))
	});
	(effective, assumed)
}

fn derive(ruleset: &Ruleset, answers: &Answers) -> BTreeMap<String, String> {
	let mut out = BTreeMap::new();
	for d in &ruleset.derivations {
		match &d.kind {
			DerivationKind::HighestBand {
				questions,
				labels,
				bump_when,
			} => {
				let mut max: Option<usize> = None;
				for qid in questions {
					if let (Some(q), Some(answer)) = (ruleset.question(qid), answers.one(qid))
						&& let Some(ix) = q.option_index(answer)
						// "I'm not sure" sits in the option list but is not a band,
						// so it must not count as the highest one reached.
						&& q.options.get(ix).is_some_and(|o| !o.unsure)
					{
						max = Some(max.map_or(ix, |m| m.max(ix)));
					}
				}
				// Bump one band up (capped at the top) when the condition holds.
				if let Some(ix) = max.as_mut()
					&& bump_when.as_ref().is_some_and(|c| c.eval(answers))
				{
					*ix = (*ix + 1).min(labels.len().saturating_sub(1));
				}
				if let Some(label) = max.and_then(|ix| labels.get(ix)) {
					out.insert(d.id.clone(), label.clone());
				}
			}
		}
	}
	out
}
