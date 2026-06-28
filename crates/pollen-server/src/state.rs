use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;

use crate::config::Config;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::ruleset::{BUNDLED_RULESET, GitHubSource, ResolvedRuleset, RulesetResolver};

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
	pub config: Arc<Config>,
	pub db: Db,
	/// The live default ruleset new drafts bind when no `?config` branch is
	/// named. Starts as the bundled default and is swapped in place when the
	/// production branch poller (see [`crate::production`]) adopts a newer one.
	pub default_ruleset: Arc<ArcSwap<ResolvedRuleset>>,
	/// Preview resolver against the configured source repository. `None` when no
	/// repository is configured, which disables `?config` previews.
	pub resolver: Option<RulesetResolver<GitHubSource>>,
}

impl AppState {
	pub async fn init() -> Result<Self> {
		let config = Config::from_env()?;
		let db = crate::db::init(&config.database_url);

		// Fail fast if the bundled ruleset can't be parsed/validated.
		let default_ruleset = Arc::new(ArcSwap::from_pointee(
			ResolvedRuleset::from_ron(BUNDLED_RULESET)
				.map_err(|e| AppError::custom(format!("bundled ruleset is invalid: {e}")))?,
		));

		let resolver = match &config.ruleset_repo {
			Some(repo) => Some(RulesetResolver::new(GitHubSource::new(
				repo.clone(),
				config.ruleset_repo_token.clone(),
			)?)),
			None => None,
		};

		// Track the production branch over the bundled default: poll it (and on
		// startup) so a ruleset change ships by pushing, without a redeploy.
		if let Some(repo) = &config.ruleset_repo
			&& config.ruleset_poll_secs > 0
		{
			let source = GitHubSource::new(repo.clone(), config.ruleset_repo_token.clone())?;
			crate::production::spawn_poller(
				source,
				db.clone(),
				config.ruleset_branch.clone(),
				Duration::from_secs(config.ruleset_poll_secs),
				default_ruleset.clone(),
			);
		}

		Ok(Self {
			config: Arc::new(config),
			db,
			default_ruleset,
			resolver,
		})
	}
}
