use std::sync::Arc;

use crate::config::Config;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::ruleset::{BUNDLED_RULESET, GitHubSource, ResolvedRuleset, RulesetResolver};

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
	pub config: Arc<Config>,
	pub db: Db,
	/// The bundled default ruleset, resolved once at boot. New drafts bind this
	/// when no `?config` branch is named — no source request is made.
	pub default_ruleset: Arc<ResolvedRuleset>,
	/// Preview resolver against the configured source repository. `None` when no
	/// repository is configured, which disables `?config` previews.
	pub resolver: Option<RulesetResolver<GitHubSource>>,
}

impl AppState {
	pub async fn init() -> Result<Self> {
		let config = Config::from_env()?;
		let db = crate::db::init(&config.database_url);

		// Fail fast if the bundled ruleset can't be parsed/validated.
		let default_ruleset = Arc::new(
			ResolvedRuleset::from_ron(BUNDLED_RULESET)
				.map_err(|e| AppError::custom(format!("bundled ruleset is invalid: {e}")))?,
		);

		let resolver = match &config.ruleset_repo {
			Some(repo) => Some(RulesetResolver::new(GitHubSource::new(
				repo.clone(),
				config.ruleset_repo_token.clone(),
			)?)),
			None => None,
		};

		Ok(Self {
			config: Arc::new(config),
			db,
			default_ruleset,
			resolver,
		})
	}
}
