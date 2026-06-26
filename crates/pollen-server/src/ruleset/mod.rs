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
pub use engine::{Evaluation, TriggeredConsequence, TriggeredGuidance, Verdict, evaluate};
pub use migrate::{Migration, migrate};
pub use model::{
	Audience, Consequence, ConsequenceType, Cost, Derivation, DerivationKind, Guidance, Opt,
	Question, QuestionKind, Rule, Ruleset, Severity, Status,
};
pub use resolver::{RULESET_PATH, ResolvedRuleset, RulesetResolver};
pub use source::{GitHubSource, RefSource};

/// The default ruleset, bundled into the binary. Bound by new drafts that name
/// no `?config` branch; resolved once at boot (see [`crate::state::AppState`]).
pub const BUNDLED_RULESET: &str =
	include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../ruleset.ron"));
