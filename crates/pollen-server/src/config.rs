use crate::error::{AppError, Result};

/// Deployment configuration. Nothing deployment-specific is hardcoded; values
/// come from the environment so the same binary runs anywhere.
#[derive(Debug, Clone)]
pub struct Config {
	/// Public base URL the tool uses to build the links it hands out (e.g. the
	/// host it is served at). Unset in local dev.
	pub public_base_url: Option<String>,

	/// Connection string for the tool's own database.
	pub database_url: String,

	/// The single source repository (`owner/repo`) whose branch refs the
	/// `?config=<branch>` preview resolves against. Unset disables previews.
	pub ruleset_repo: Option<String>,

	/// Optional token to lift the source host's unauthenticated rate limit.
	pub ruleset_repo_token: Option<String>,
}

impl Config {
	pub fn from_env() -> Result<Self> {
		Ok(Self {
			public_base_url: std::env::var("PUBLIC_BASE_URL").ok(),
			database_url: std::env::var("DATABASE_URL")
				.map_err(|_| AppError::custom("DATABASE_URL must be set"))?,
			ruleset_repo: std::env::var("RULESET_REPO").ok(),
			ruleset_repo_token: std::env::var("RULESET_REPO_TOKEN").ok(),
		})
	}
}
