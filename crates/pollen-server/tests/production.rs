//! The production-ruleset poller adopts a changed, valid ruleset from the
//! tracked branch, ignores an unchanged one, and leaves the current default in
//! place when the branch's ruleset is invalid.

mod common;

use arc_swap::ArcSwap;
use common::run_db;
use pollen_server::db::ConfigRow;
use pollen_server::error::Result;
use pollen_server::production::refresh_default;
use pollen_server::ruleset::{BUNDLED_RULESET, RefSource, ResolvedRuleset};

/// A source that returns canned ruleset content for any ref.
#[derive(Clone)]
struct FakeSource {
	content: String,
}

impl RefSource for FakeSource {
	async fn resolve_ref(&self, _branch: &str) -> Result<String> {
		Ok("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string())
	}
	async fn fetch_file(&self, _commit: &str, _path: &str) -> Result<String> {
		Ok(self.content.clone())
	}
}

#[tokio::test(flavor = "multi_thread")]
async fn adopts_a_changed_production_ruleset() {
	run_db(|db, mut conn| async move {
		let current = ArcSwap::from_pointee(ResolvedRuleset::from_ron(BUNDLED_RULESET).unwrap());
		let bundled_hash = current.load().hash.clone();

		// A real content change (a tweaked label) from the tracked branch.
		let modified = BUNDLED_RULESET.replace("Connect to Tupaia?", "Connect to Tupaia (prod)?");
		assert_ne!(
			modified, BUNDLED_RULESET,
			"the replacement must change content"
		);
		let source = FakeSource { content: modified };

		// First refresh adopts it: the default swaps and the content is stored.
		let new_hash = refresh_default(&source, &db, "main", &current)
			.await
			.unwrap()
			.expect("adopted a new hash");
		assert_ne!(new_hash, bundled_hash);
		assert_eq!(current.load().hash, new_hash);
		assert!(ConfigRow::get(&mut conn, &new_hash).await.is_ok());

		// A second refresh with the same content is a no-op.
		assert!(
			refresh_default(&source, &db, "main", &current)
				.await
				.unwrap()
				.is_none()
		);
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_production_ruleset_is_ignored() {
	run_db(|db, _conn| async move {
		let current = ArcSwap::from_pointee(ResolvedRuleset::from_ron(BUNDLED_RULESET).unwrap());
		let before = current.load().hash.clone();

		let source = FakeSource {
			content: "this is not valid RON".to_string(),
		};
		assert!(
			refresh_default(&source, &db, "main", &current)
				.await
				.is_err()
		);
		// The current default is left untouched.
		assert_eq!(current.load().hash, before);
	})
	.await;
}
