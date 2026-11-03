//! The hosting cost model against the reference fixtures, and the internal
//! pricing endpoints: gating, the calculator, and pricing a plan (spec COST).

mod common;

use std::collections::BTreeMap;

use axum::http::StatusCode;
use common::{PRICING_PATH, run_server};
use pollen_server::pricing::{BUNDLED_PRICING, CloneMode, CostInput, PricingConfig};
use serde_json::{Value, json};

const FIXTURES: &str = include_str!("fixtures/pricing_cases.json");

/// A fixture input, with the reference model's defaults for anything omitted.
fn input_from_fixture(v: &Value) -> CostInput {
	let band = v["band"].as_str().unwrap();
	let mut name = band.to_string();
	name[..1].make_ascii_uppercase();
	CostInput {
		band: name,
		central_in_cloud: v["central"].as_str().unwrap_or("bescloud") == "bescloud",
		cloud_facilities: v["cloud_facilities"].as_u64().unwrap_or(0) as u32,
		clone: match v["clone"].as_str().unwrap_or("standard") {
			"off" => CloneMode::Off,
			"standard" => CloneMode::Standard,
			"always_on" => CloneMode::AlwaysOn,
			other => panic!("unknown clone mode {other}"),
		},
		clone_uptime: None,
		backups: v["backups"].as_bool().unwrap_or(true),
		tupaia: v["tupaia"].as_bool().unwrap_or(false),
		windows_central: v["windows_central"].as_bool().unwrap_or(false),
		windows_facilities: v["windows_facilities"].as_u64().unwrap_or(0) as u32,
		windows_clone: v["windows_clone"].as_bool().unwrap_or(false),
	}
}

fn assert_cents(what: &str, actual: f64, expected: f64) {
	assert!(
		(actual - expected).abs() < 0.0051,
		"{what}: got {actual:.4}, expected {expected:.2}"
	);
}

#[test]
fn matches_every_reference_case_to_the_cent() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let fixtures: Value = serde_json::from_str(FIXTURES).unwrap();
	let examples = fixtures["examples"].as_array().unwrap();
	let grid = fixtures["grid"].as_array().unwrap();
	assert_eq!(examples.len() + grid.len(), 297);

	for (i, case) in examples.iter().chain(grid).enumerate() {
		let name = case["name"]
			.as_str()
			.map_or(format!("case {i}"), str::to_string);
		let input = input_from_fixture(&case["input"]);
		let got = config
			.cost(&input)
			.unwrap_or_else(|e| panic!("{name}: {e}"));
		let expected = &case["expected"];

		assert_cents(
			&format!("{name} monthly"),
			got.monthly_usd,
			expected["monthly_usd"].as_f64().unwrap(),
		);
		assert_cents(
			&format!("{name} annual"),
			got.annual_usd,
			expected["annual_usd"].as_f64().unwrap(),
		);

		let lines: BTreeMap<String, f64> = got
			.lines
			.iter()
			.map(|l| {
				(
					serde_json::to_value(l.id)
						.unwrap()
						.as_str()
						.unwrap()
						.to_string(),
					l.monthly_usd,
				)
			})
			.collect();
		let want: BTreeMap<String, f64> = expected["lines"]
			.as_object()
			.unwrap()
			.iter()
			.map(|(k, v)| (k.clone(), v.as_f64().unwrap()))
			.collect();
		assert_eq!(
			lines.keys().collect::<Vec<_>>(),
			want.keys().collect::<Vec<_>>(),
			"{name}: line set"
		);
		for (k, v) in &want {
			assert_cents(&format!("{name} {k}"), lines[k], *v);
		}
		// The breakdown is what the figure is made of.
		let sum: f64 = got.lines.iter().map(|l| l.monthly_usd).sum();
		assert_cents(&format!("{name} sum"), sum, got.monthly_usd);
	}
}

#[test]
fn backups_off_leaves_the_clone_alone() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let on = input_from_fixture(&json!({ "band": "medium" }));
	let off = CostInput {
		backups: false,
		..on.clone()
	};
	let a = config.cost(&on).unwrap();
	let b = config.cost(&off).unwrap();
	assert_cents(
		"difference",
		a.monthly_usd - b.monthly_usd,
		config.backups_share(),
	);
	assert!(
		b.lines
			.iter()
			.any(|l| l.id == pollen_server::pricing::LineId::Clone)
	);
}

#[test]
fn windows_needs_a_cloud_box_to_cost_anything() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	// A client-hosted Windows central, Windows facilities beyond the cloud
	// count, and a Windows clone with no clone all cost BES nothing extra.
	let input = input_from_fixture(&json!({
		"band": "small", "central": "onprem", "clone": "off",
		"windows_central": true, "windows_facilities": 3, "windows_clone": true,
	}));
	let got = config.cost(&input).unwrap();
	assert!(
		got.lines
			.iter()
			.all(|l| l.id != pollen_server::pricing::LineId::WindowsExtra)
	);
}

#[test]
fn clone_uptime_scales_only_compute() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let base = input_from_fixture(&json!({ "band": "small" }));
	let at = |u: f64| {
		config
			.cost(&CostInput {
				clone_uptime: Some(u),
				..base.clone()
			})
			.unwrap()
			.monthly_usd
	};
	// Always-on equals a standard clone at 100%.
	let always = config
		.cost(&CostInput {
			clone: CloneMode::AlwaysOn,
			..base.clone()
		})
		.unwrap()
		.monthly_usd;
	assert_cents("100%", at(1.0), always);
	assert_cents(
		"30% default",
		at(0.3),
		config.cost(&base).unwrap().monthly_usd,
	);
	assert!(
		config
			.cost(&CostInput {
				clone_uptime: Some(1.5),
				..base
			})
			.is_err()
	);
}

#[test]
fn display_rounds_to_the_configured_step() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let got = config
		.cost(&input_from_fixture(&json!({ "band": "small" })))
		.unwrap();
	// 242.65 a month, 2,911.79 a year.
	assert_eq!(got.display.monthly, 245.0);
	assert_eq!(got.display.annual, 2910.0);
	assert_eq!(got.display.currency, "USD");
}

// ── HTTP ────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn endpoints_hide_behind_the_pricing_path() {
	run_server(|server, _conn| async move {
		for fn_ in ["model", "cost", "plan"] {
			let path = format!("/api/pricing/{fn_}");
			server
				.post(&path)
				.json(&json!({}))
				.await
				.assert_status(StatusCode::NOT_FOUND);
			server
				.post(&path)
				.add_header("x-pricing-path", "test-pricing-path-012345678X")
				.json(&json!({}))
				.await
				.assert_status(StatusCode::NOT_FOUND);
		}
		let model: Value = server
			.post("/api/pricing/model")
			.add_header("x-pricing-path", PRICING_PATH)
			.json(&json!({}))
			.await
			.json();
		assert_eq!(model["bands"].as_array().unwrap().len(), 4);
		assert_eq!(model["params"].as_array().unwrap().len(), 14);
		assert_eq!(model["params"][0]["confidence"], "Read");
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn calculator_prices_inputs() {
	run_server(|server, _conn| async move {
		let got: Value = server
			.post("/api/pricing/cost")
			.add_header("x-pricing-path", PRICING_PATH)
			.json(&json!({
				"band": "Medium", "central_in_cloud": true, "cloud_facilities": 2,
				"clone": "standard", "backups": true, "tupaia": true,
			}))
			.await
			.json();
		assert_cents("monthly", got["monthly_usd"].as_f64().unwrap(), 1077.57);

		server
			.post("/api/pricing/cost")
			.add_header("x-pricing-path", PRICING_PATH)
			.json(&json!({
				"band": "Enterprise", "central_in_cloud": true, "cloud_facilities": 0,
				"clone": "off", "backups": true, "tupaia": false,
			}))
			.await
			.assert_status(StatusCode::BAD_REQUEST);
	})
	.await;
}

async fn plan_cost(server: &axum_test::TestServer, answers: Value, overrides: Value) -> Value {
	let created: Value = server
		.post("/api/applications/create")
		.json(&json!({}))
		.await
		.json();
	let id = created["id"].as_str().unwrap().to_owned();
	server
		.post("/api/applications/patch")
		.json(&json!({ "id": id, "answers": answers }))
		.await
		.assert_status_ok();
	server
		.post("/api/pricing/plan")
		.add_header("x-pricing-path", PRICING_PATH)
		.json(&json!({ "id": id, "overrides": overrides }))
		.await
		.json()
}

#[tokio::test(flavor = "multi_thread")]
async fn plan_reads_its_answers() {
	run_server(|server, _conn| async move {
		// Unsized: no figure, and no lightest-band guess.
		let unsized_plan = plan_cost(&server, json!({}), json!({})).await;
		assert!(unsized_plan["cost"].is_null());
		assert!(
			unsized_plan["missing"]
				.as_str()
				.unwrap()
				.contains("catchment")
		);

		// Medium, everything else assumed: central and facilities are the
		// ruleset's defaults (BES cloud central, client-hosted facilities),
		// Tupaia and backups default on, and the clone is standard.
		let medium = plan_cost(
			&server,
			json!({ "catchment": "c2", "facilities": "f0" }),
			json!({}),
		)
		.await;
		assert_eq!(medium["size"], "Medium");
		assert_eq!(medium["guess"]["count"], 0);
		assert_cents(
			"medium",
			medium["cost"]["monthly_usd"].as_f64().unwrap(),
			939.27,
		);

		// All facility servers in BES cloud: 2-5 facilities guesses three.
		let cloud = plan_cost(
			&server,
			json!({ "catchment": "c0", "facilities": "f1", "hosting_where": "allbes", "tupaia": "no" }),
			json!({}),
		)
		.await;
		assert_eq!(cloud["guess"]["count"], 3);
		assert_eq!(cloud["input"]["cloud_facilities"], 3);
		assert_eq!(cloud["input"]["tupaia"], false);

		// Overrides change the figure without touching the plan.
		let overridden = plan_cost(
			&server,
			json!({ "catchment": "c0", "facilities": "f1", "hosting_where": "allbes", "tupaia": "no" }),
			json!({ "cloud_facilities": 1, "clone": "off" }),
		)
		.await;
		assert_eq!(overridden["guess"]["count"], 3);
		assert_eq!(overridden["input"]["cloud_facilities"], 1);
		assert_cents(
			"override",
			overridden["cost"]["monthly_usd"].as_f64().unwrap(),
			227.83,
		);

		// Client-hosted central; Tupaia left open is noted, and left out.
		let open = plan_cost(
			&server,
			json!({ "catchment": "c0", "facilities": "f0", "central": "clienthosted", "tupaia": "unsure" }),
			json!({}),
		)
		.await;
		assert_eq!(open["input"]["central_in_cloud"], false);
		assert_eq!(open["open"][0]["id"], "tupaia_reporting");
		assert_eq!(open["open"][0]["included"], false);

		// Backups off drops only the backups share.
		let no_backups = plan_cost(
			&server,
			json!({ "catchment": "c0", "facilities": "f0", "tupaia": "no", "backup_capability": "no" }),
			json!({}),
		)
		.await;
		assert_eq!(no_backups["input"]["backups"], false);
		assert_eq!(no_backups["input"]["clone"], "standard");
		assert_cents(
			"no backups",
			no_backups["cost"]["monthly_usd"].as_f64().unwrap(),
			204.12,
		);
	})
	.await;
}
