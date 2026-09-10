//! The ruleset: its data model, the condition language, the evaluation engine,
//! and normalization/hashing. The ruleset is authored in RON (see `ruleset/`)
//! and is the data the engine evaluates (spec WIZ).

pub mod answers;
pub mod condition;
pub mod engine;
pub mod migrate;
pub mod model;
pub mod normalize;
pub mod resolver;
pub mod source;

pub use answers::{Answer, Answers};
pub use condition::Condition;
pub use engine::{
	Assumed, Evaluation, TriggeredConsequence, TriggeredGuidance, TriggeredRequirement, Verdict,
	evaluate,
};
pub use migrate::{Migration, migrate};
pub use model::{
	Audience, Consequence, ConsequenceType, Cost, Derivation, DerivationKind, Guidance, Opt,
	Question, QuestionKind, Requirement, Rule, Ruleset, Section, Severity, SizeSpecs, Spec,
	SpecRow, Status,
};
pub use resolver::{RULESET_PATH, ResolvedRuleset, RulesetResolver};
pub use source::{GitHubSource, RefSource};

/// The default ruleset, bundled into the binary. Bound by new drafts that name
/// no `?config` branch; resolved once at boot (see [`crate::state::AppState`]).
pub const BUNDLED_RULESET: &str =
	include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../ruleset.ron"));

#[cfg(test)]
mod tests {
	use super::*;

	/// The bundled ruleset is what every new draft binds, so a mistake in it
	/// breaks the tool at boot rather than at review. Parse it, run the stable-id
	/// checks, and verify the authoring invariants the engine relies on.
	#[test]
	fn bundled_ruleset_is_well_formed() {
		let ruleset = Ruleset::from_ron(BUNDLED_RULESET).expect("bundled ruleset parses");
		ruleset.validate().expect("bundled ruleset validates");

		let sections: Vec<&str> = ruleset.sections.iter().map(|s| s.id.as_str()).collect();
		assert!(!sections.is_empty(), "the flow declares its sections");

		for q in &ruleset.questions {
			if let Some(section) = &q.section {
				assert!(
					sections.contains(&section.as_str()),
					"question {} names section {section}, which is not declared",
					q.id
				);
			}
			// A default is applied verbatim as an answer, so it must name a real
			// option, and assuming "I'm not sure" would be a contradiction.
			if let Some(default) = &q.default {
				let opt = q
					.options
					.iter()
					.find(|o| &o.id == default)
					.unwrap_or_else(|| {
						panic!("question {} defaults to unknown option {default}", q.id)
					});
				assert!(
					!opt.unsure,
					"question {} defaults to its unsure option, which would assume an unknown",
					q.id
				);
			}
			// Every question must be resolvable without an answer, or it blocks
			// finalising. Only the sizing bands are allowed to demand one.
			let resolvable = q.default.is_some() || q.options.iter().any(|o| o.unsure);
			assert!(
				resolvable || matches!(q.id.as_str(), "catchment" | "facilities"),
				"question {} can neither be defaulted nor declined, so it would block finalising",
				q.id
			);
		}
	}
}
