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
		tupaia: v["tupaia"].as_bool().unwrap_or(true),
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

/// A fixture's parameter override, applied to a copy of the configuration.
fn config_with(overrides: &Value) -> PricingConfig {
	let mut config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	for (name, v) in overrides.as_object().into_iter().flatten() {
		let v = v.as_f64().unwrap();
		match name.as_str() {
			"pgro_ops_share" => config.params.pgro_ops_share.value = v,
			other => panic!("fixture overrides {other}, which the harness does not know"),
		}
	}
	config
}

#[test]
fn matches_every_reference_case_to_the_cent() {
	let fixtures: Value = serde_json::from_str(FIXTURES).unwrap();
	let examples = fixtures["examples"].as_array().unwrap();
	let grid = fixtures["grid"].as_array().unwrap();
	assert_eq!(examples.len() + grid.len(), 299);

	for (i, case) in examples.iter().chain(grid).enumerate() {
		let name = case["name"]
			.as_str()
			.map_or(format!("case {i}"), str::to_string);
		let config = config_with(&case["params_override"]);
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

		// What is displayed adds up: the headline is the sum of the shown
		// lines, and the annual figure is twelve times it.
		assert_eq!(
			got.display.lines.len(),
			got.lines.len(),
			"{name}: display lines"
		);
		let shown: f64 = got.display.lines.iter().sum();
		assert_cents(&format!("{name} displayed sum"), shown, got.display.monthly);
		assert_cents(
			&format!("{name} displayed annual"),
			got.display.monthly * 12.0,
			got.display.annual,
		);
		// And it stays close to the cost: each line is within half a step.
		assert!((got.display.monthly - got.monthly_usd).abs() <= 2.5 * got.lines.len() as f64);
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
	assert_cents("difference", a.monthly_usd - b.monthly_usd, {
		let db = config.band("Medium").unwrap().db_gb;
		config.backups_share(db) + config.s3_backup_share(db)
	});
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
fn lines_carry_the_weakest_confidence_they_rest_on() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let got = config
		.cost(&input_from_fixture(
			&json!({ "band": "medium", "tupaia": true }),
		))
		.unwrap();
	let tag = |id: pollen_server::pricing::LineId| {
		serde_json::to_value(got.lines.iter().find(|l| l.id == id).unwrap().confidence).unwrap()
	};
	use pollen_server::pricing::LineId::*;
	// Rates read from price lists.
	assert_eq!(tag(CentralBox), "Read");
	// Allocations of shared accounts.
	assert_eq!(tag(PlatformShare), "Estimated");
	assert_eq!(tag(Clone), "Estimated");
	assert_eq!(tag(TupaiaReporting), "Estimated");
	// A share of the data cluster, by an estimated database size.
	assert_eq!(tag(BackupsOpsShare), "Estimated");
}

#[test]
fn refuses_more_facility_servers_than_it_will_price() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let base = input_from_fixture(&json!({ "band": "small" }));
	let max = config.max_cloud_facilities;
	assert!(
		config
			.cost(&CostInput {
				cloud_facilities: max,
				..base.clone()
			})
			.is_ok()
	);
	assert!(
		config
			.cost(&CostInput {
				cloud_facilities: max + 1,
				..base.clone()
			})
			.is_err()
	);
	// A clone that never runs reads as 0%, not -0%.
	let off = config
		.cost(&CostInput {
			clone_uptime: Some(-0.0),
			..base
		})
		.unwrap();
	let clone = off
		.lines
		.iter()
		.find(|l| l.id == pollen_server::pricing::LineId::Clone)
		.unwrap();
	assert_eq!(
		clone.detail.as_deref(),
		Some("Compute 0% of the month, storage always")
	);
}

#[test]
fn shows_storage_with_a_thousands_separator() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let got = config
		.cost(&input_from_fixture(&json!({ "band": "large" })))
		.unwrap();
	let central = got
		.lines
		.iter()
		.find(|l| l.id == pollen_server::pricing::LineId::CentralBox)
		.unwrap();
	assert!(
		central
			.detail
			.as_deref()
			.unwrap()
			.ends_with("1,500\u{a0}GB")
	);
}

#[test]
fn matches_the_figures_the_model_update_publishes() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	// Central in BES cloud, clone, backups: without and with Tupaia.
	for (band, plain, with_tupaia) in [
		("tiny", 195.0, 282.0),
		("small", 200.0, 309.0),
		("medium", 542.0, 846.0),
		("large", 1017.0, 1447.0),
	] {
		let base = input_from_fixture(&json!({ "band": band, "tupaia": false }));
		let tupaia = CostInput {
			tupaia: true,
			..base.clone()
		};
		assert_eq!(
			config.cost(&base).unwrap().monthly_usd.round(),
			plain,
			"{band}"
		);
		assert_eq!(
			config.cost(&tupaia).unwrap().monthly_usd.round(),
			with_tupaia,
			"{band}"
		);
	}
	// Backup storage per band, from S3 usage by database size.
	for (band, usd) in [
		("Tiny", 0.60),
		("Small", 3.01),
		("Medium", 24.11),
		("Large", 37.68),
	] {
		let db = config.band(band).unwrap().db_gb;
		assert_cents(band, config.s3_backup_share(db), usd);
	}
}

#[test]
fn backup_storage_follows_the_backups_share_and_goes_with_it() {
	use pollen_server::pricing::LineId;
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let on = config
		.cost(&input_from_fixture(&json!({ "band": "medium" })))
		.unwrap();
	let ids: Vec<LineId> = on.lines.iter().map(|l| l.id).collect();
	let ops = ids
		.iter()
		.position(|i| *i == LineId::BackupsOpsShare)
		.unwrap();
	assert_eq!(
		ids[ops + 1],
		LineId::BackupStorage,
		"straight after the ops share"
	);
	let off = config
		.cost(&input_from_fixture(
			&json!({ "band": "medium", "backups": false }),
		))
		.unwrap();
	assert!(
		off.lines
			.iter()
			.all(|l| l.id != LineId::BackupStorage && l.id != LineId::BackupsOpsShare)
	);
}

#[test]
fn display_rounds_to_the_configured_step() {
	let config = PricingConfig::from_ron(BUNDLED_PRICING).unwrap();
	let got = config
		.cost(&input_from_fixture(&json!({ "band": "small" })))
		.unwrap();
	// 309.05 a month. Each line rounds to $5 (80, 0, 85, 30, 5, 5, 110) and the
	// headline is their sum, so it adds up on the page.
	assert_eq!(
		got.display.lines,
		vec![80.0, 0.0, 85.0, 30.0, 5.0, 5.0, 110.0]
	);
	assert_eq!(got.display.monthly, 315.0);
	assert_eq!(got.display.annual, 3780.0);
	assert_eq!(got.display.currency, "USD");
}

// ── HTTP ────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn endpoints_hide_behind_the_pricing_path() {
	run_server(|server, _conn| async move {
		for fn_ in ["model", "cost", "size", "plan"] {
			let path = format!("/api/pricing/{fn_}");
			server
				.post(&path)
				.json(&json!({}))
				.await
				.assert_status(StatusCode::NOT_FOUND);
			server
				.post(&path)
				.add_header("x-pricing-path", format!("{}x", PRICING_PATH.as_str()))
				.json(&json!({}))
				.await
				.assert_status(StatusCode::NOT_FOUND);
		}
		let model: Value = server
			.post("/api/pricing/model")
			.add_header("x-pricing-path", PRICING_PATH.as_str())
			.json(&json!({}))
			.await
			.json();
		assert_eq!(model["bands"].as_array().unwrap().len(), 4);
	})
	.await;
}

#[tokio::test(flavor = "multi_thread")]
async fn calculator_prices_inputs() {
	run_server(|server, _conn| async move {
		let got: Value = server
			.post("/api/pricing/cost")
			.add_header("x-pricing-path", PRICING_PATH.as_str())
			.json(&json!({
				"band": "Medium", "central_in_cloud": true, "cloud_facilities": 2,
				"clone": "standard", "backups": true, "tupaia": true,
			}))
			.await
			.json();
		assert_cents("monthly", got["monthly_usd"].as_f64().unwrap(), 984.31);

		server
			.post("/api/pricing/cost")
			.add_header("x-pricing-path", PRICING_PATH.as_str())
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
		.add_header("x-pricing-path", PRICING_PATH.as_str())
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
				.contains("first two questions")
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
			846.01,
		);

		// Connected systems do not move the band for pricing, although the
		// ruleset's own size counts them.
		let integrated = plan_cost(
			&server,
			json!({ "catchment": "c0", "facilities": "f0", "integrations": ["lims"] }),
			json!({}),
		)
		.await;
		assert_eq!(integrated["size"], "Tiny");

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

		// A mix of both is half the all-cloud guess: 6-20 facilities guesses ten.
		let mix = plan_cost(
			&server,
			json!({ "catchment": "c0", "facilities": "f2", "hosting_where": "mix", "tupaia": "no" }),
			json!({}),
		)
		.await;
		assert_eq!(mix["guess"]["count"], 5);
		assert_eq!(mix["guess"]["total"], 10);

		// A setting can put no more in BES cloud than there are facility servers.
		let capped = plan_cost(
			&server,
			json!({ "catchment": "c0", "facilities": "f1", "hosting_where": "mix", "tupaia": "no" }),
			json!({ "cloud_facilities": 15 }),
		)
		.await;
		assert_eq!(capped["guess"]["total"], 3);
		assert_eq!(capped["input"]["cloud_facilities"], 3);

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
			184.99,
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
		// Shown as the breakdown shows it: $86.93 for a Tiny to the nearest step.
		assert_eq!(open["open"][0]["display_monthly"], 85.0);

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
			194.12,
		);
	})
	.await;
}

async fn size(server: &axum_test::TestServer, body: Value) -> Value {
	server
		.post("/api/pricing/size")
		.add_header("x-pricing-path", PRICING_PATH.as_str())
		.json(&body)
		.await
		.json()
}

#[tokio::test(flavor = "multi_thread")]
async fn estimator_derives_the_band_the_wizard_would() {
	run_server(|server, _conn| async move {
		// Nothing answered: no band to offer.
		let none = size(&server, json!({})).await;
		assert!(none["band"].is_null());
		assert!(none["cloud_facilities"].is_null());

		// The highest band reached across the sizing questions wins.
		let got = size(
			&server,
			json!({ "answers": { "catchment": "c0", "facilities": "f2" } }),
		)
		.await;
		assert_eq!(got["band"], "Medium");
		assert_eq!(got["cloud_facilities"], 10);

		// "I'm not sure" contributes nothing, and connected systems do not
		// move the band.
		let got = size(
			&server,
			json!({ "answers": { "catchment": "c1", "mobile": "m_unsure" } }),
		)
		.await;
		assert_eq!(got["band"], "Small");
		assert!(got["cloud_facilities"].is_null());

		// Unknown ids are ignored.
		let got = size(
			&server,
			json!({ "answers": { "catchment": "c3", "facilities": "nope", "region": "sydney" } }),
		)
		.await;
		assert_eq!(got["band"], "Large");

		// The model carries what the estimator asks, and a typical count per band.
		let model: Value = server
			.post("/api/pricing/model")
			.add_header("x-pricing-path", PRICING_PATH.as_str())
			.json(&json!({}))
			.await
			.json();
		assert_eq!(model["estimator"]["sizing"].as_array().unwrap().len(), 3);
		assert_eq!(model["facility_guess_by_band"], json!([1, 3, 10, 20]));
		assert_eq!(model["mix_share"], 0.5);
		assert_eq!(model["facility_slider_question"], "hosting_balance");
	})
	.await;
}
