//! The v1 ruleset parses, validates, hashes deterministically, and evaluates to
//! the expected verdicts/consequences for representative configurations.

use pollen_server::ruleset::{
	Answers, Ruleset, Verdict, evaluate,
	normalize::{canonical_json, content_hash},
};
use serde_json::json;

const RULESET: &str = include_str!("../../../ruleset.ron");

fn v1() -> Ruleset {
	Ruleset::from_ron(RULESET).expect("parse ruleset.ron")
}

fn answers(value: serde_json::Value) -> Answers {
	serde_json::from_value(value).expect("answers")
}

fn fired_ids(eval: &pollen_server::ruleset::Evaluation) -> Vec<&str> {
	eval.consequences.iter().map(|c| c.id.as_str()).collect()
}

#[test]
fn v1_parses_and_validates() {
	let ruleset = v1();
	ruleset
		.validate()
		.expect("v1 obeys the stable-id discipline");
	assert!(!ruleset.questions.is_empty());
	assert!(!ruleset.rules.is_empty());
}

#[test]
fn canonical_hash_is_deterministic() {
	let a = canonical_json(&v1()).unwrap();
	let b = canonical_json(&v1()).unwrap();
	assert_eq!(a, b);
	assert_eq!(content_hash(&a), content_hash(&b));
	// sha-256 hex.
	assert_eq!(content_hash(&a).len(), 64);
}

#[test]
fn demo_config_is_blocking() {
	// The prototype's demo config: Tupaia on, backups disabled, Windows,
	// other AWS region, hybrid cloud + on-prem, infrequent upgrades.
	let eval = evaluate(
		&v1(),
		&answers(json!({
			"tupaia": "yes",
			"integrations": ["lims"],
			"hosted_integration": "no",
			"catchment": "c1",
			"facilities": "f2",
			"mobile": "m2",
			"central": "bescloud",
			"facility_mix": ["bescloud", "baremetal", "iti"],
			"region": "otheraws",
			"platform": "windows",
			"backup_capability": "no",
			"cadence": "biannual",
			"dns": "client",
			"remote": "other",
			"timesync": "outbound",
		})),
	);

	assert_eq!(eval.verdict, Verdict::Blocking);
	// Bands reach Medium; the LIMS integration bumps it to Large.
	assert_eq!(eval.derived.get("size").map(String::as_str), Some("Large"));

	let ids = fired_ids(&eval);
	// The blocking conflict and a representative spread of consequences.
	for expected in [
		"block-backup-tupaia",
		"backup-off",
		"tupaia-on",
		"int-capacity",
		"region-other",
		"plat-windows",
		"prov-baremetal",
		"iti-note",
		"dns-client",
		"remote-other",
		"dns-partition",
		"egress-ip",
	] {
		assert!(
			ids.contains(&expected),
			"expected {expected} to fire; got {ids:?}"
		);
	}
	// Not fired: the client hosts integrations; no virtualized facilities.
	assert!(!ids.contains(&"int-hosted"));
	assert!(!ids.contains(&"prov-virtualized"));
}

#[test]
fn tupaia_without_backups_blocks() {
	let eval = evaluate(
		&v1(),
		&answers(json!({ "tupaia": "yes", "backup_capability": "no" })),
	);
	assert_eq!(eval.verdict, Verdict::Blocking);
	assert!(fired_ids(&eval).contains(&"block-backup-tupaia"));
}

#[test]
fn default_path_is_clear() {
	// All-cloud, full backups, BES-controlled networking: every triggered
	// consequence is default-severity, so the verdict stays clear.
	let eval = evaluate(
		&v1(),
		&answers(json!({
			"tupaia": "no",
			"integrations": ["none"],
			"catchment": "c0",
			"facilities": "f0",
			"mobile": "m0",
			"central": "bescloud",
			"facility_mix": ["bescloud"],
			"region": "sydney",
			"backup_capability": "yes",
			"retention": "full",
			"cadence": "release",
			"dns": "bes",
			"dns_arrangement": "bes_subdomain",
			"remote": "tailscale",
			"timesync": "internal",
		})),
	);

	assert_eq!(eval.verdict, Verdict::Clear);
	assert_eq!(eval.derived.get("size").map(String::as_str), Some("Tiny"));
	let ids = fired_ids(&eval);
	assert!(ids.contains(&"dns-bes-subdomain"));
	assert!(!ids.contains(&"region-other"));
	assert!(!ids.contains(&"plat-windows"));
}

#[test]
fn non_fhir_integration_adds_cost_without_blocking() {
	let eval = evaluate(
		&v1(),
		&answers(json!({ "integrations": ["other_nonfhir"] })),
	);
	let ids = fired_ids(&eval);
	assert!(ids.contains(&"int-nonfhir-cost"));
	assert!(ids.contains(&"int-capacity"));
	// A cost, not a blocker.
	assert_eq!(eval.verdict, Verdict::NonDefault);
}

#[test]
fn none_integration_is_exclusive() {
	let rs = v1();
	let none = rs
		.question("integrations")
		.unwrap()
		.options
		.iter()
		.find(|o| o.id == "none")
		.unwrap();
	assert!(none.exclusive);
}

#[test]
fn integrations_bump_the_size() {
	// Bands reach Medium either way; an integration bumps it to Large.
	let bumped = evaluate(
		&v1(),
		&answers(json!({ "catchment": "c2", "integrations": ["lims"] })),
	);
	assert_eq!(
		bumped.derived.get("size").map(String::as_str),
		Some("Large")
	);

	let plain = evaluate(
		&v1(),
		&answers(json!({ "catchment": "c2", "integrations": ["none"] })),
	);
	assert_eq!(
		plain.derived.get("size").map(String::as_str),
		Some("Medium")
	);
}

#[test]
fn no_dns_is_an_off_default_risk() {
	let eval = evaluate(&v1(), &answers(json!({ "dns": "local" })));
	assert!(fired_ids(&eval).contains(&"dns-local"));
	assert_eq!(eval.verdict, Verdict::NonDefault);
}

#[test]
fn self_hosted_central_with_mobile_needs_public_ip() {
	// Mobile clients + client-hosted Central → public-IP requirement; not when
	// BES hosts Central, and not when there are no mobile clients.
	let onprem = evaluate(
		&v1(),
		&answers(json!({ "central": "onprem", "mobile": "m2" })),
	);
	assert!(fired_ids(&onprem).contains(&"mobile-public-ip"));

	let bes = evaluate(
		&v1(),
		&answers(json!({ "central": "bescloud", "mobile": "m2" })),
	);
	assert!(!fired_ids(&bes).contains(&"mobile-public-ip"));

	let no_mobile = evaluate(
		&v1(),
		&answers(json!({ "central": "onprem", "mobile": "m0" })),
	);
	assert!(!fired_ids(&no_mobile).contains(&"mobile-public-ip"));
}

#[test]
fn on_prem_requires_network_setup() {
	let onprem = evaluate(&v1(), &answers(json!({ "facility_mix": ["baremetal"] })));
	assert!(fired_ids(&onprem).contains(&"onprem-network"));

	let cloud = evaluate(&v1(), &answers(json!({ "facility_mix": ["bescloud"] })));
	assert!(!fired_ids(&cloud).contains(&"onprem-network"));
}

#[test]
fn declining_telemetry_is_an_off_default_opt_out() {
	let off = evaluate(&v1(), &answers(json!({ "telemetry": "no" })));
	assert!(fired_ids(&off).contains(&"telemetry-off"));
	assert_eq!(off.verdict, Verdict::NonDefault);

	// The outbound allowance only applies when there are on-prem servers.
	let on = evaluate(
		&v1(),
		&answers(json!({ "telemetry": "yes", "central": "onprem" })),
	);
	let ids = fired_ids(&on);
	assert!(ids.contains(&"telemetry-on"));
	assert!(!ids.contains(&"telemetry-off"));
}

#[test]
fn client_network_items_need_on_prem() {
	let common = |facility: &str| {
		json!({
			"central": "bescloud",
			"facility_mix": [facility],
			"remote": "tailscale",
			"timesync": "outbound",
			"telemetry": "yes",
		})
	};
	// All-cloud: the client-side network allowances don't apply.
	let cloud = evaluate(&v1(), &answers(common("bescloud")));
	let cloud_ids = fired_ids(&cloud);
	for id in ["remote-tailscale", "time-outbound", "telemetry-on"] {
		assert!(!cloud_ids.contains(&id), "{id} should not fire all-cloud");
	}
	// With an on-prem facility, they do.
	let onprem = evaluate(&v1(), &answers(common("baremetal")));
	let onprem_ids = fired_ids(&onprem);
	for id in ["remote-tailscale", "time-outbound", "telemetry-on"] {
		assert!(onprem_ids.contains(&id), "{id} should fire with on-prem");
	}
}

#[test]
fn declining_telemetry_blocks_tupaia_and_mobile() {
	let tupaia = evaluate(
		&v1(),
		&answers(json!({ "telemetry": "no", "tupaia": "yes" })),
	);
	assert!(fired_ids(&tupaia).contains(&"block-telemetry-tupaia"));
	assert_eq!(tupaia.verdict, Verdict::Blocking);

	let mobile = evaluate(
		&v1(),
		&answers(json!({ "telemetry": "no", "mobile": "m2" })),
	);
	assert!(fired_ids(&mobile).contains(&"block-telemetry-mobile"));
	assert_eq!(mobile.verdict, Verdict::Blocking);
}

#[test]
fn iti_only_hides_the_on_prem_os_question() {
	let shows = |eval: &pollen_server::ruleset::Evaluation| {
		eval.visible_questions.iter().any(|q| q == "platform")
	};
	// BES cloud + ITI only: both are fixed to Linux/ARM64, so no OS to choose.
	let iti = evaluate(
		&v1(),
		&answers(json!({ "central": "bescloud", "facility_mix": ["bescloud", "iti"] })),
	);
	assert!(!shows(&iti));
	// A bare-metal facility does have an OS to choose.
	let baremetal = evaluate(
		&v1(),
		&answers(json!({ "central": "bescloud", "facility_mix": ["baremetal"] })),
	);
	assert!(shows(&baremetal));
}

#[test]
fn windows_requires_time_sync_setup() {
	// Windows servers don't get time sync for free the way the Linux servers do,
	// so choosing Windows always raises the requirement to configure it.
	let eval = evaluate(&v1(), &answers(json!({ "platform": "windows" })));
	assert!(fired_ids(&eval).contains(&"time-windows"));
}

#[test]
fn dns_arrangement_targets_the_consequence() {
	// Each arrangement fires its own consequence. Only the SOA-delegated client
	// subdomain reads as off-default; the rest stay on the default path.
	let subdomain = evaluate(
		&v1(),
		&answers(json!({ "dns": "bes", "dns_arrangement": "bes_subdomain" })),
	);
	assert!(fired_ids(&subdomain).contains(&"dns-bes-subdomain"));
	assert_eq!(subdomain.verdict, Verdict::Clear);

	// A client-owned domain pointed at BES is targeted but not off-default.
	let client_domain = evaluate(
		&v1(),
		&answers(json!({ "dns": "bes", "dns_arrangement": "client_domain" })),
	);
	let ids = fired_ids(&client_domain);
	assert!(ids.contains(&"dns-bes-client-domain"));
	assert!(!ids.contains(&"dns-bes-subdomain"));
	assert_eq!(client_domain.verdict, Verdict::Clear);

	// Only the SOA delegation is off-default.
	let soa = evaluate(
		&v1(),
		&answers(json!({ "dns": "bes", "dns_arrangement": "client_subdomain" })),
	);
	assert!(fired_ids(&soa).contains(&"dns-bes-client-subdomain"));
	assert_eq!(soa.verdict, Verdict::NonDefault);
}

#[test]
fn tupaia_guidance_shows_at_backups() {
	let eval = evaluate(&v1(), &answers(json!({ "tupaia": "yes" })));
	assert!(
		eval.guidance
			.iter()
			.any(|g| g.at == "backup_capability" && g.message.contains("low-retention"))
	);
}
