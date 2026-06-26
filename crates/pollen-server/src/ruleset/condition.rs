//! The declarative condition language that drives triggering, visibility, and
//! forward guidance (spec WIZ, Triggering). Conditions are evaluated against a
//! user's answers and express presence-of-class and cross-field checks.

use serde::{Deserialize, Serialize};

use super::answers::Answers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
	/// Always true (the default visibility for a question).
	Always,
	/// The question has any answer.
	Answered(String),
	/// A single-select / band question's answer equals an option id.
	Equals(String, String),
	/// A multi-select question's answers include an option id.
	Includes(String, String),
	/// All sub-conditions hold (vacuously true when empty).
	All(Vec<Condition>),
	/// Any sub-condition holds (false when empty).
	Any(Vec<Condition>),
	/// The sub-condition does not hold.
	Not(Box<Condition>),
}

impl Condition {
	/// The default visibility condition.
	pub fn always() -> Self {
		Condition::Always
	}

	pub fn eval(&self, answers: &Answers) -> bool {
		match self {
			Condition::Always => true,
			Condition::Answered(q) => answers.answered(q),
			Condition::Equals(q, opt) => answers.one(q) == Some(opt.as_str()),
			Condition::Includes(q, opt) => answers.many(q).iter().any(|o| o == opt),
			Condition::All(cs) => cs.iter().all(|c| c.eval(answers)),
			Condition::Any(cs) => cs.iter().any(|c| c.eval(answers)),
			Condition::Not(c) => !c.eval(answers),
		}
	}
}
