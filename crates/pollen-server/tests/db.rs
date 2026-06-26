mod common;

use common::TestDb;
use pollen_server::db::{Application, ApplicationStatus, ConfigRow};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn config_store_and_application_roundtrip() {
	TestDb::run(|mut conn| async move {
		let content = json!({ "questions": [], "rules": [] });
		let hash = "deadbeef";

		// First store, then a duplicate store: the second dedupes to one row.
		ConfigRow::upsert(&mut conn, hash, &content).await.unwrap();
		ConfigRow::upsert(&mut conn, hash, &content).await.unwrap();
		let stored = ConfigRow::get(&mut conn, hash).await.unwrap();
		assert_eq!(stored.config_hash, hash);
		assert_eq!(stored.content, content);

		// A new draft bound to that ruleset, with the expected defaults.
		let draft = Application::create_draft(&mut conn, hash, None, &json!({}))
			.await
			.unwrap();
		assert_eq!(draft.status, ApplicationStatus::Draft);
		assert_eq!(draft.config_hash, hash);
		assert_eq!(draft.answers, json!({}));
		assert!(draft.parent_id.is_none());
		assert!(draft.finalized_at.is_none());

		// Round-trips by id.
		let fetched = Application::get(&mut conn, draft.id).await.unwrap();
		assert_eq!(fetched.id, draft.id);

		// A fork records lineage back to its predecessor.
		let child = Application::create_draft(&mut conn, hash, Some(draft.id), &json!({}))
			.await
			.unwrap();
		assert_eq!(child.parent_id, Some(draft.id));
	})
	.await;
}
