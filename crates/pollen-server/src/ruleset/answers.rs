//! A user's answers: a map from question id to the option(s) chosen. Stored as
//! jsonb on the application row and deserialized into these types for
//! evaluation. Answers are keyed by stable id, never by position or label.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Answers(pub BTreeMap<String, Answer>);

/// A single-select / band answer is one option id; a multi-select is many.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Answer {
	One(String),
	Many(Vec<String>),
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

	/// Whether the question has any answer at all.
	pub fn answered(&self, question: &str) -> bool {
		match self.0.get(question) {
			Some(Answer::One(v)) => !v.is_empty(),
			Some(Answer::Many(v)) => !v.is_empty(),
			None => false,
		}
	}
}
