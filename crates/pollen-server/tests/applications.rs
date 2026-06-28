//! HTTP lifecycle: create → patch → finalise → fork, plus the draft/finalised
//! mutability rules and preview-not-configured handling.

mod common;

use axum::http::StatusCode;
use common::run_server;
use pollen_server::db::{Application, ConfigRow};
use pollen_server::ruleset::{BUNDLED_RULESET, ResolvedRuleset};
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
		// Freshly bound to the bundled default, so no update is available.
		assert_eq!(created["update_available"], false);
		let id = created["id"].as_str().unwrap().to_owned();

		// A complete, blocking configuration: Tupaia on but backups disabled.
		// Every visible question is answered (so finalise is allowed); platform,
		// retention, and hosted-integration stay hidden here.
		let answers = json!({
			"tupaia": "yes",
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
			"dns_arrangement": "bes_subdomain",
			"remote": "tailscale",
			"timesync": "internal",
			"telemetry": "yes",
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
		assert_eq!(forked["answers"]["tupaia"], "yes");
		assert_eq!(forked["answers"]["central"], "bescloud");
		assert_eq!(forked["evaluation"]["verdict"], "Blocking");
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn fork_to_default_updates_a_stale_draft() {
	run_server(|server, mut conn| async move {
		// A draft bound to a ruleset hash other than the current default. We
		// store the bundled ruleset's content under a fabricated hash so it
		// still evaluates, but reads as "not the current default".
		let bundled = ResolvedRuleset::from_ron(BUNDLED_RULESET).expect("bundled ruleset");
		let content = serde_json::to_value(&bundled.ruleset).unwrap();
		let stale_hash = "stale-test-hash";
		ConfigRow::upsert(&mut conn, stale_hash, &content)
			.await
			.unwrap();
		let app =
			Application::create_draft(&mut conn, stale_hash, None, &json!({ "tupaia": "yes" }))
				.await
				.unwrap();
		let id = app.id.to_string();

		// The draft reports an update is available (bound hash != default).
		let fetched: Value = server
			.post("/api/applications/get")
			.json(&json!({ "id": id }))
			.await
			.json();
		assert_eq!(fetched["update_available"], true);
		assert_eq!(fetched["config_hash"], stale_hash);

		// Updating to the default rebinds, carries answers over, and clears the
		// flag; lineage points back to the stale draft.
		let updated: Value = server
			.post("/api/applications/fork")
			.json(&json!({ "id": id, "to_default": true }))
			.await
			.json();
		assert_eq!(updated["status"], "draft");
		assert_eq!(updated["parent_id"], id);
		assert_eq!(updated["update_available"], false);
		assert_ne!(updated["config_hash"], stale_hash);
		assert_eq!(updated["answers"]["tupaia"], "yes");
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn stale_finalised_plan_also_offers_an_update() {
	run_server(|server, mut conn| async move {
		// A finalised plan frozen against a now-superseded ruleset still surfaces
		// that a newer default is available (it would fork to a new draft).
		let bundled = ResolvedRuleset::from_ron(BUNDLED_RULESET).expect("bundled ruleset");
		let content = serde_json::to_value(&bundled.ruleset).unwrap();
		let stale_hash = "stale-finalised-hash";
		ConfigRow::upsert(&mut conn, stale_hash, &content)
			.await
			.unwrap();
		let app =
			Application::create_draft(&mut conn, stale_hash, None, &json!({ "tupaia": "yes" }))
				.await
				.unwrap();
		Application::finalise(&mut conn, app.id).await.unwrap();

		let fetched: Value = server
			.post("/api/applications/get")
			.json(&json!({ "id": app.id.to_string() }))
			.await
			.json();
		assert_eq!(fetched["status"], "finalised");
		assert_eq!(fetched["update_available"], true);
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
			.json(&json!({ "id": id, "answers": { "tupaia": "yes" } }))
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
			.json(&json!({ "id": id, "answers": { "tupaia": 7 } }))
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
