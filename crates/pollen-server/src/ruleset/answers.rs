//! A user's answers: a map from question id to the option(s) chosen. Stored as
//! jsonb on the application row and deserialized into these types for
//! evaluation. Answers are keyed by stable id, never by position or label.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Answers(pub BTreeMap<String, Answer>);

/// A single-select / band answer is one option id; a multi-select is many; a
/// mix is a percentage share per option id.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Answer {
	One(String),
	Many(Vec<String>),
	Mix(BTreeMap<String, u32>),
}

impl Answers {
	/// The chosen option id for a single-select or band question.
	pub fn one(&self, question: &str) -> Option<&str> {
		match self.0.get(question) {
			Some(Answer::One(v)) => Some(v.as_str()),
			_ => None,
		}
	}

	/// The chosen option ids for a multi-select question (empty if unanswered).
	pub fn many(&self, question: &str) -> &[String] {
		match self.0.get(question) {
			Some(Answer::Many(v)) => v.as_slice(),
			_ => &[],
		}
	}

	/// The percentage share an option holds in a mix question (0 when absent).
	pub fn share(&self, question: &str, option: &str) -> u32 {
		match self.0.get(question) {
			Some(Answer::Mix(m)) => m.get(option).copied().unwrap_or(0),
			_ => 0,
		}
	}

	/// Whether the question has any answer at all.
	pub fn answered(&self, question: &str) -> bool {
		match self.0.get(question) {
			Some(Answer::One(v)) => !v.is_empty(),
			Some(Answer::Many(v)) => !v.is_empty(),
			// A mix with every share at zero says nothing, so it isn't an answer.
			Some(Answer::Mix(m)) => m.values().any(|v| *v > 0),
			None => false,
		}
	}

	/// Record an answer, used to apply assumed defaults before evaluation.
	pub fn set(&mut self, question: &str, answer: Answer) {
		self.0.insert(question.to_string(), answer);
	}
}
