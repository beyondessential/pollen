//! Normalize a ruleset to its canonical JSON form and hash it. The canonical
//! form is the machine representation stored in `config_store` and is the
//! ruleset's content-addressed identity (spec WIZ, Content-addressed binding).

use sha2::{Digest, Sha256};

use super::model::Ruleset;
use crate::error::{AppError, Result};

/// Canonical JSON for a ruleset. `serde_json`'s object maps are key-sorted, so
/// this is deterministic for a given ruleset regardless of how the RON source
/// was formatted or commented.
pub fn canonical_json(ruleset: &Ruleset) -> Result<String> {
	let value = serde_json::to_value(ruleset).map_err(AppError::custom)?;
	serde_json::to_string(&value).map_err(AppError::custom)
}

/// The sha-256 hex digest of a canonical JSON string. This is the `config_hash`.
pub fn content_hash(canonical_json: &str) -> String {
	let mut hasher = Sha256::new();
	hasher.update(canonical_json.as_bytes());
	hex::encode(hasher.finalize())
}
