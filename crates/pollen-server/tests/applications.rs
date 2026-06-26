//! HTTP lifecycle: create → patch → finalize → fork, plus the draft/finalized
//! mutability rules and preview-not-configured handling.

mod common;

use axum::http::StatusCode;
use common::run_server;
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn create_patch_finalize_fork_lifecycle() {
	run_server(|server, _conn| async move {
		// Create a draft (binds the bundled default; no answers yet).
		let created: Value = server
			.post("/api/applications/create")
			.json(&json!({}))
			.await
			.json();
		assert_eq!(created["status"], "draft");
		assert_eq!(created["evaluation"]["verdict"], "Clear");
		let id = created["id"].as_str().unwrap().to_owned();

		// Patch in a blocking combination: analytics on, backups disabled.
		let patched: Value = server
			.post("/api/applications/patch")
			.json(
				&json!({ "id": id, "answers": { "analytics": "yes", "backup_capability": "no" } }),
			)
			.await
			.json();
		assert_eq!(patched["status"], "draft");
		assert_eq!(patched["evaluation"]["verdict"], "Blocking");

		// Get round-trips the same state.
		let fetched: Value = server
			.post("/api/applications/get")
			.json(&json!({ "id": id }))
			.await
			.json();
		assert_eq!(fetched["evaluation"]["verdict"], "Blocking");

		// Finalize freezes it (the blocking verdict is recorded, not enforced).
		let finalized: Value = server
			.post("/api/applications/finalize")
			.json(&json!({ "id": id }))
			.await
			.json();
		assert_eq!(finalized["status"], "finalized");
		assert!(finalized["finalized_at"].is_string());

		// A finalized artifact is immutable: patch and finalize both 409.
		server
			.post("/api/applications/patch")
			.json(&json!({ "id": id, "answers": {} }))
			.await
			.assert_status(StatusCode::CONFLICT);
		server
			.post("/api/applications/finalize")
			.json(&json!({ "id": id }))
			.await
			.assert_status(StatusCode::CONFLICT);

		// Fork spawns a new draft with lineage back to the predecessor; same
		// ruleset, so nothing migrates.
		let forked: Value = server
			.post("/api/applications/fork")
			.json(&json!({ "id": id }))
			.await
			.json();
		assert_eq!(forked["status"], "draft");
		assert_eq!(forked["parent_id"], id);
		assert_ne!(forked["id"], id);
		assert_eq!(forked["migration"]["dropped"], json!([]));
		assert_eq!(forked["migration"]["new_questions"], json!([]));
		// Carried answers re-evaluate to the same verdict.
		assert_eq!(forked["evaluation"]["verdict"], "Blocking");
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn get_missing_application_is_404() {
	run_server(|server, _conn| async move {
		server
			.post("/api/applications/get")
			.json(&json!({ "id": "00000000-0000-0000-0000-000000000000" }))
			.await
			.assert_status(StatusCode::NOT_FOUND);
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_answers_are_rejected() {
	run_server(|server, _conn| async move {
		let created: Value = server
			.post("/api/applications/create")
			.json(&json!({}))
			.await
			.json();
		let id = created["id"].as_str().unwrap().to_owned();
		// A number is not a valid answer (option id or list of them).
		server
			.post("/api/applications/patch")
			.json(&json!({ "id": id, "answers": { "analytics": 7 } }))
			.await
			.assert_status(StatusCode::BAD_REQUEST);
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn preview_without_a_configured_repo_is_rejected() {
	run_server(|server, _conn| async move {
		// The test server configures no source repo, so a branch preview can't
		// be served.
		server
			.post("/api/applications/create")
			.json(&json!({ "config_branch": "some-branch" }))
			.await
			.assert_status(StatusCode::BAD_REQUEST);
	})
	.await;
}
