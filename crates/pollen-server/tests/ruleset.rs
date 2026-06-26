//! The v1 ruleset parses, validates, hashes deterministically, and evaluates to
//! the expected verdicts/consequences for representative configurations.

use pollen_server::ruleset::{
	Answers, Ruleset, Verdict, evaluate,
	normalize::{canonical_json, content_hash},
};
use serde_json::json;

const V1: &str = include_str!("../../../ruleset/v1.ron");

fn v1() -> Ruleset {
	Ruleset::from_ron(V1).expect("parse v1.ron")
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
	// The prototype's demo config: analytics on, backups disabled, Windows,
	// other AWS region, hybrid cloud + on-prem, infrequent upgrades.
	let eval = evaluate(
		&v1(),
		&answers(json!({
			"analytics": "yes",
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
	assert_eq!(eval.derived.get("size").map(String::as_str), Some("Medium"));

	let ids = fired_ids(&eval);
	// The blocking conflict and a representative spread of consequences.
	for expected in [
		"block-backup-analytics",
		"backup-off",
		"analytics-on",
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
fn analytics_without_backups_blocks() {
	let eval = evaluate(
		&v1(),
		&answers(json!({ "analytics": "yes", "backup_capability": "no" })),
	);
	assert_eq!(eval.verdict, Verdict::Blocking);
	assert!(fired_ids(&eval).contains(&"block-backup-analytics"));
}

#[test]
fn default_path_is_clear() {
	// All-cloud, full backups, BES-controlled networking: every triggered
	// consequence is default-severity, so the verdict stays clear.
	let eval = evaluate(
		&v1(),
		&answers(json!({
			"analytics": "no",
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
			"remote": "tailscale",
			"timesync": "internal",
		})),
	);

	assert_eq!(eval.verdict, Verdict::Clear);
	assert_eq!(eval.derived.get("size").map(String::as_str), Some("Small"));
	let ids = fired_ids(&eval);
	assert!(ids.contains(&"dns-bes"));
	assert!(!ids.contains(&"region-other"));
	assert!(!ids.contains(&"plat-windows"));
}

#[test]
fn analytics_guidance_shows_at_backups() {
	let eval = evaluate(&v1(), &answers(json!({ "analytics": "yes" })));
	assert!(
		eval.guidance
			.iter()
			.any(|g| g.at == "backup_capability" && g.message.contains("low-retention"))
	);
}
