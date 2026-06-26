//! HTTP lifecycle: create → patch → finalise → fork, plus the draft/finalised
//! mutability rules and preview-not-configured handling.

mod common;

use axum::http::StatusCode;
use common::run_server;
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn create_patch_finalise_fork_lifecycle() {
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

		// A complete, blocking configuration: analytics on but backups disabled.
		// Every visible question is answered (so finalise is allowed); platform,
		// retention, and hosted-integration stay hidden here.
		let answers = json!({
			"analytics": "yes",
			"integrations": ["none"],
			"catchment": "c0",
			"facilities": "f0",
			"mobile": "m0",
			"central": "bescloud",
			"facility_mix": ["bescloud"],
			"region": "sydney",
			"backup_capability": "no",
			"cadence": "release",
			"dns": "bes",
			"remote": "tailscale",
			"timesync": "internal",
		});
		let patched: Value = server
			.post("/api/applications/patch")
			.json(&json!({ "id": id, "answers": answers }))
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

		// Finalise freezes it (the blocking verdict is recorded, not enforced).
		let finalised: Value = server
			.post("/api/applications/finalise")
			.json(&json!({ "id": id }))
			.await
			.json();
		assert_eq!(finalised["status"], "finalised");
		assert!(finalised["finalised_at"].is_string());

		// A finalised artifact is immutable: patch and finalise both 409.
		server
			.post("/api/applications/patch")
			.json(&json!({ "id": id, "answers": {} }))
			.await
			.assert_status(StatusCode::CONFLICT);
		server
			.post("/api/applications/finalise")
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
		// Answers carry over to the new draft (not a zeroed plan).
		assert_eq!(forked["answers"]["analytics"], "yes");
		assert_eq!(forked["answers"]["central"], "bescloud");
		assert_eq!(forked["evaluation"]["verdict"], "Blocking");
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn finalise_requires_all_questions_answered() {
	run_server(|server, _conn| async move {
		let created: Value = server
			.post("/api/applications/create")
			.json(&json!({}))
			.await
			.json();
		let id = created["id"].as_str().unwrap().to_owned();

		// Only one of many visible questions answered.
		server
			.post("/api/applications/patch")
			.json(&json!({ "id": id, "answers": { "analytics": "yes" } }))
			.await
			.assert_status_ok();

		// Finalising an incomplete plan is rejected.
		server
			.post("/api/applications/finalise")
			.json(&json!({ "id": id }))
			.await
			.assert_status(StatusCode::BAD_REQUEST);
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
