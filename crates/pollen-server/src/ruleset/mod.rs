//! The ruleset: its data model, the condition language, the evaluation engine,
//! and normalization/hashing. The ruleset is authored in RON (see `ruleset/`)
//! and is the data the engine evaluates (spec WIZ).

pub mod answers;
pub mod condition;
pub mod engine;
pub mod model;
pub mod normalize;

pub use answers::{Answer, Answers};
pub use condition::Condition;
pub use engine::{Evaluation, TriggeredConsequence, TriggeredGuidance, Verdict, evaluate};
pub use model::{
	Audience, Consequence, ConsequenceType, Cost, Derivation, DerivationKind, Guidance, Opt,
	Question, QuestionKind, Rule, Ruleset, Severity, Status,
};
