//! Tracks the production ruleset on the configured repo's branch (default
//! `main`): when it carries a different, valid ruleset than the running default,
//! the daemon adopts it. So a ruleset change ships by pushing to the branch,
//! without rebuilding the engine. The bundled ruleset stays the fallback — an
//! unreachable branch or an invalid ruleset leaves the current default in place.

use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;

use crate::db::{ConfigRow, Db};
use crate::error::{AppError, Result};
use crate::ruleset::{RULESET_PATH, RefSource, ResolvedRuleset};

/// Resolve the branch, fetch and validate its ruleset, and adopt it as the
/// default when it differs from the current one. Returns the adopted hash, or
/// `None` if the branch already matches. Any error leaves the current default
/// untouched (the caller logs and retries on the next tick).
pub async fn refresh_default<S: RefSource>(
	source: &S,
	db: &Db,
	branch: &str,
	current: &ArcSwap<ResolvedRuleset>,
) -> Result<Option<String>> {
	let commit = source.resolve_ref(branch).await?;
	let content = source.fetch_file(&commit, RULESET_PATH).await?;
	let resolved = ResolvedRuleset::from_ron(&content)?;
	if resolved.hash == current.load().hash {
		return Ok(None);
	}
	// Persist the content so plans bound to the new hash can load it.
	let mut conn = db.get().await?;
	let json = serde_json::to_value(&resolved.ruleset).map_err(AppError::custom)?;
	ConfigRow::upsert(&mut conn, &resolved.hash, &json).await?;
	let hash = resolved.hash.clone();
	current.store(Arc::new(resolved));
	Ok(Some(hash))
}

/// Spawn a background task that refreshes immediately, then every `interval`.
pub fn spawn_poller<S: RefSource>(
	source: S,
	db: Db,
	branch: String,
	interval: Duration,
	current: Arc<ArcSwap<ResolvedRuleset>>,
) {
	tokio::spawn(async move {
		// The first tick fires immediately, so this also covers the startup check.
		let mut ticker = tokio::time::interval(interval);
		loop {
			ticker.tick().await;
			match refresh_default(&source, &db, &branch, &current).await {
				Ok(Some(hash)) => tracing::info!(%branch, %hash, "adopted production ruleset"),
				Ok(None) => tracing::debug!(%branch, "production ruleset unchanged"),
				Err(e) => {
					tracing::warn!(%branch, error = %e, "production ruleset refresh failed; keeping current");
				}
			}
		}
	});
}
