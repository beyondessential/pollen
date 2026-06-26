//! Stable-id migration: move an artifact's answers from one bound ruleset to
//! another by comparing question ids (spec WIZ, Stable-id migration). The diff
//! is the "what changed" summary the user sees on update.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::answers::Answers;
use super::model::Ruleset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
	/// Answers carried over: their question id is present in the new ruleset.
	pub answers: Answers,
	/// Question ids whose answers were dropped — no longer in the new ruleset.
	pub dropped: Vec<String>,
	/// Question ids new in the new ruleset — they appear unanswered.
	pub new_questions: Vec<String>,
}

/// Diff answers across a ruleset change. A question id present in both carries
/// its answer over (a changed label or consequence is re-evaluation, not a
/// migration concern); one removed drops its answer; one newly present appears
/// unanswered.
pub fn migrate(old: &Ruleset, new: &Ruleset, answers: &Answers) -> Migration {
	let old_ids: BTreeSet<&str> = old.questions.iter().map(|q| q.id.as_str()).collect();
	let new_ids: BTreeSet<&str> = new.questions.iter().map(|q| q.id.as_str()).collect();

	let mut carried = BTreeMap::new();
	let mut dropped = Vec::new();
	for (qid, answer) in &answers.0 {
		if new_ids.contains(qid.as_str()) {
			carried.insert(qid.clone(), answer.clone());
		} else {
			dropped.push(qid.clone());
		}
	}
	dropped.sort();

	let new_questions = new
		.questions
		.iter()
		.filter(|q| !old_ids.contains(q.id.as_str()))
		.map(|q| q.id.clone())
		.collect();

	Migration {
		answers: Answers(carried),
		dropped,
		new_questions,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	fn ruleset_with(question_ids: &[&str]) -> Ruleset {
		let questions: String = question_ids
			.iter()
			.map(|id| format!("(id: \"{id}\", kind: Single, label: \"{id}\"),"))
			.collect();
		Ruleset::from_ron(&format!("(questions: [{questions}], rules: [])")).unwrap()
	}

	#[test]
	fn carries_drops_and_flags_new_questions() {
		let old = ruleset_with(&["a", "b"]);
		let new = ruleset_with(&["b", "c"]);
		let answers: Answers = serde_json::from_value(json!({ "a": "x", "b": "y" })).unwrap();

		let migration = migrate(&old, &new, &answers);

		assert_eq!(migration.dropped, vec!["a"]);
		assert_eq!(migration.new_questions, vec!["c"]);
		assert_eq!(migration.answers.one("b"), Some("y"));
		assert_eq!(migration.answers.one("a"), None);
	}

	#[test]
	fn identical_rulesets_carry_everything() {
		let rs = ruleset_with(&["a", "b"]);
		let answers: Answers = serde_json::from_value(json!({ "a": "x", "b": "y" })).unwrap();

		let migration = migrate(&rs, &rs, &answers);

		assert!(migration.dropped.is_empty());
		assert!(migration.new_questions.is_empty());
		assert_eq!(migration.answers.one("a"), Some("x"));
	}
}
