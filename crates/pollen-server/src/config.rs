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

	/// The production branch the daemon tracks: when it carries a different,
	/// valid ruleset than the bundled default, that becomes the live default.
	pub ruleset_branch: String,

	/// How often to poll the production branch, in seconds (the first check runs
	/// at startup). `0` disables polling — the bundled default is used as-is.
	pub ruleset_poll_secs: u64,
}

impl Config {
	pub fn from_env() -> Result<Self> {
		Ok(Self {
			public_base_url: std::env::var("PUBLIC_BASE_URL").ok(),
			database_url: std::env::var("DATABASE_URL")
				.map_err(|_| AppError::custom("DATABASE_URL must be set"))?,
			ruleset_repo: std::env::var("RULESET_REPO").ok(),
			ruleset_repo_token: std::env::var("RULESET_REPO_TOKEN").ok(),
			ruleset_branch: std::env::var("RULESET_BRANCH").unwrap_or_else(|_| "main".to_string()),
			ruleset_poll_secs: std::env::var("RULESET_POLL_SECS")
				.ok()
				.and_then(|s| s.parse().ok())
				.unwrap_or(300),
		})
	}
}
