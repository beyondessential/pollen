//! The v1 ruleset parses, validates, hashes deterministically, and evaluates to
//! the expected verdicts/consequences for representative configurations,
//! including the escape hatches a non-technical user relies on: assumed
//! defaults, "I'm not sure" open items, and the questions that still must be
//! answered.

use pollen_server::ruleset::{
	Answers, Ruleset, Severity, Verdict, evaluate,
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

/// The three sizing bands, so a test can focus on what it's actually asserting
/// without leaving the required questions unanswered.
fn sized() -> serde_json::Value {
	json!({ "catchment": "c0", "facilities": "f0", "mobile": "m0" })
}

/// Merge extra answers over a base object.
fn with(base: serde_json::Value, extra: serde_json::Value) -> Answers {
	let mut base = base.as_object().cloned().expect("object");
	for (k, v) in extra.as_object().expect("object") {
		base.insert(k.clone(), v.clone());
	}
	answers(serde_json::Value::Object(base))
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
	// Tupaia on but backups disabled, Windows, another AWS region, a hybrid
	// cloud/client-hosted/Iti split, infrequent upgrades.
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
			"hosting_where": "mix",
			"hosting_balance": "half",
			"iti_use": "some",
			"onprem_form": "baremetal",
			"region": "otheraws",
			"platform": "windows",
			"backup_capability": "no",
			"cadence": "biannual",
			"dns": "client",
			"remote": "other",
			"timesync": "outbound",
			"telemetry": "yes",
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
	// Not fired: the client hosts integrations; the servers aren't virtualised.
	assert!(!ids.contains(&"int-hosted"));
	assert!(!ids.contains(&"prov-virtualised"));
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
			"hosting_where": "allbes",
			"region": "sydney",
			"backup_capability": "yes",
			"retention": "full",
			"cadence": "release",
			"dns": "bes",
			"dns_arrangement": "bes_subdomain",
			"remote": "tailscale",
			"timesync": "internal",
			"telemetry": "yes",
		})),
	);

	assert_eq!(eval.verdict, Verdict::Clear);
	assert_eq!(eval.derived.get("size").map(String::as_str), Some("Tiny"));
	let ids = fired_ids(&eval);
	assert!(ids.contains(&"dns-bes-subdomain"));
	assert!(!ids.contains(&"region-other"));
	assert!(!ids.contains(&"plat-windows"));
	// Nothing was guessed and nothing is outstanding.
	assert!(eval.assumed.is_empty(), "assumed: {:?}", eval.assumed);
	assert!(eval.open_items.is_empty());
	assert!(eval.required.is_empty());
}

// ── The escape hatches ──────────────────────────────────────────────────────

#[test]
fn only_the_sizing_bands_must_be_answered() {
	// An empty plan: catchment and facilities are the whole of what's required,
	// so a non-technical user is never blocked on a question they can't answer.
	let eval = evaluate(&v1(), &answers(json!({})));
	assert_eq!(eval.required, vec!["catchment", "facilities"]);

	// Once those two are given, nothing blocks finalising.
	let eval = evaluate(
		&v1(),
		&answers(json!({ "catchment": "c1", "facilities": "f1" })),
	);
	assert!(
		eval.required.is_empty(),
		"still required: {:?}",
		eval.required
	);
}

#[test]
fn unanswered_questions_take_their_blessed_default() {
	// Answer only the essentials; everything with a default is filled in.
	let eval = evaluate(&v1(), &answers(sized()));
	let assumed: Vec<(&str, &str)> = eval
		.assumed
		.iter()
		.map(|a| (a.question.as_str(), a.option.as_str()))
		.collect();

	for expected in [
		("tupaia", "yes"),
		("hosting_where", "allclient"),
		("central", "bescloud"),
		("platform", "linuxarm"),
		("remote", "tailscale"),
		("timesync", "outbound"),
		("cadence", "twomonths"),
		("backup_capability", "yes"),
		("telemetry", "yes"),
	] {
		assert!(
			assumed.contains(&expected),
			"expected {expected:?} to be assumed; got {assumed:?}"
		);
	}
	// The assumptions are live: the blessed platform's consequence fires.
	assert!(fired_ids(&eval).contains(&"plat-image"));
}

#[test]
fn a_default_can_reveal_a_question_that_is_itself_defaulted() {
	// hosting_where defaults to all client hosted, which reveals the Iti,
	// provisioning and OS questions, which have defaults of their own. A single
	// pass would miss them, so the engine runs defaults to a fixed point.
	let eval = evaluate(&v1(), &answers(sized()));
	let assumed: Vec<&str> = eval.assumed.iter().map(|a| a.question.as_str()).collect();
	assert!(assumed.contains(&"hosting_where"));
	assert!(
		assumed.contains(&"onprem_form"),
		"a question revealed by an assumption should be assumed too; got {assumed:?}"
	);
	assert!(assumed.contains(&"platform"));
}

#[test]
fn dns_ownership_is_never_guessed() {
	// Who owns the domain varies too much between clients to assume, so it
	// records as an open item rather than taking a default.
	let eval = evaluate(&v1(), &answers(sized()));
	assert!(
		eval.open_items.iter().any(|o| o == "dns"),
		"dns should be an open item; got {:?}",
		eval.open_items
	);
	assert!(!eval.assumed.iter().any(|a| a.question == "dns"));
}

#[test]
fn the_untouched_default_path_raises_nothing() {
	// Answering only the essentials must not produce a callout: everything
	// assumed is on the blessed path, so the rail stays empty.
	let eval = evaluate(&v1(), &answers(sized()));
	assert_eq!(eval.verdict, Verdict::Clear);
	let flagged: Vec<&str> = eval
		.consequences
		.iter()
		.filter(|c| c.consequence.severity != Severity::Default)
		.map(|c| c.id.as_str())
		.collect();
	assert!(flagged.is_empty(), "unexpected callouts: {flagged:?}");
	// The upgrade advisory is for cadences slower than the default.
	assert!(!fired_ids(&eval).contains(&"cadence"));
}

#[test]
fn choosing_unsure_records_an_open_item_and_asserts_nothing() {
	let eval = evaluate(&v1(), &with(sized(), json!({ "telemetry": "unsure" })));
	assert!(eval.open_items.iter().any(|o| o == "telemetry"));
	// Neither the "allowed" nor the "declined" consequence may fire off an unknown.
	let ids = fired_ids(&eval);
	assert!(!ids.contains(&"telemetry-off"));
	assert!(!ids.contains(&"telemetry-on"));
}

#[test]
fn an_unsure_multi_select_is_an_open_item() {
	// The unsure detection has to reach into multi-selects, not just single ones.
	let eval = evaluate(&v1(), &with(sized(), json!({ "integrations": ["unsure"] })));
	assert!(eval.open_items.iter().any(|o| o == "integrations"));
	assert!(!fired_ids(&eval).contains(&"int-capacity"));
}

#[test]
fn an_unsure_band_does_not_size_the_deployment() {
	// "I'm not sure" sits last in the option list, so a naive ordinal read would
	// make it the highest band and either inflate or wipe the size.
	let eval = evaluate(
		&v1(),
		&answers(json!({ "catchment": "c1", "facilities": "f0", "mobile": "m_unsure" })),
	);
	assert_eq!(eval.derived.get("size").map(String::as_str), Some("Small"));
	assert!(eval.open_items.iter().any(|o| o == "mobile"));
}

#[test]
fn an_unsure_answer_hides_the_questions_it_gates() {
	// Declining the DNS question suppresses the follow-up rather than asking it
	// against an unknown.
	let asked = evaluate(&v1(), &with(sized(), json!({ "dns": "bes" })));
	assert!(
		asked
			.visible_questions
			.iter()
			.any(|q| q == "dns_arrangement")
	);

	let unsure = evaluate(&v1(), &with(sized(), json!({ "dns": "unsure" })));
	assert!(
		!unsure
			.visible_questions
			.iter()
			.any(|q| q == "dns_arrangement")
	);
	assert!(
		!unsure
			.assumed
			.iter()
			.any(|a| a.question == "dns_arrangement")
	);
}

#[test]
fn an_unsure_mobile_count_does_not_assert_mobile_clients() {
	// Rules keyed on "mobile is in play" must not fire off an unknown.
	let eval = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "mobile": "m_unsure", "telemetry": "no", "tupaia": "no", "central": "clienthosted" }),
		),
	);
	let ids = fired_ids(&eval);
	assert!(!ids.contains(&"block-telemetry-mobile"));
	assert!(!ids.contains(&"mobile-public-ip"));
}

// ── Consequences ────────────────────────────────────────────────────────────

#[test]
fn non_fhir_integration_adds_cost_without_blocking() {
	let eval = evaluate(
		&v1(),
		&with(sized(), json!({ "integrations": ["other_nonfhir"] })),
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
	let eval = evaluate(&v1(), &with(sized(), json!({ "dns": "local" })));
	assert!(fired_ids(&eval).contains(&"dns-local"));
	assert_eq!(eval.verdict, Verdict::NonDefault);
}

#[test]
fn amd64_is_supported_but_carries_a_penalty() {
	// ARM64 is what BES supports; AMD64 is allowed but must read as off-default.
	let arm = evaluate(&v1(), &with(sized(), json!({ "platform": "linuxarm" })));
	assert!(!fired_ids(&arm).contains(&"plat-amd64"));

	let amd = evaluate(&v1(), &with(sized(), json!({ "platform": "linuxamd" })));
	assert!(fired_ids(&amd).contains(&"plat-amd64"));
	assert_eq!(amd.verdict, Verdict::NonDefault);
}

#[test]
fn client_hosted_central_is_off_default() {
	let eval = evaluate(&v1(), &with(sized(), json!({ "central": "clienthosted" })));
	assert!(fired_ids(&eval).contains(&"central-clienthosted"));
	assert_eq!(eval.verdict, Verdict::NonDefault);
}

#[test]
fn self_hosted_central_with_mobile_needs_public_ip() {
	// Mobile clients + client-hosted Central → public-IP requirement; not when
	// BES hosts Central, and not when there are no mobile clients.
	let onprem = evaluate(
		&v1(),
		&answers(json!({ "central": "clienthosted", "mobile": "m2" })),
	);
	assert!(fired_ids(&onprem).contains(&"mobile-public-ip"));

	let bes = evaluate(
		&v1(),
		&answers(json!({ "central": "bescloud", "mobile": "m2" })),
	);
	assert!(!fired_ids(&bes).contains(&"mobile-public-ip"));

	let no_mobile = evaluate(
		&v1(),
		&answers(json!({ "central": "clienthosted", "mobile": "m0" })),
	);
	assert!(!fired_ids(&no_mobile).contains(&"mobile-public-ip"));
}

#[test]
fn on_prem_requires_network_setup() {
	let onprem = evaluate(
		&v1(),
		&with(sized(), json!({ "hosting_where": "allclient" })),
	);
	assert!(fired_ids(&onprem).contains(&"onprem-network"));

	let cloud = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "central": "bescloud", "hosting_where": "allbes" }),
		),
	);
	assert!(!fired_ids(&cloud).contains(&"onprem-network"));
}

#[test]
fn an_all_cloud_deployment_asks_nothing_about_client_servers() {
	// With everything in BES cloud there is no Iti question, no OS to choose,
	// and none of the client-network requirements.
	let eval = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "central": "bescloud", "hosting_where": "allbes" }),
		),
	);
	for q in ["iti_use", "onprem_form", "platform"] {
		assert!(
			!eval.visible_questions.iter().any(|v| v == q),
			"{q} should be hidden when everything is in BES cloud"
		);
	}
	let ids = fired_ids(&eval);
	assert!(!ids.contains(&"onprem-network"));
	assert!(!ids.contains(&"iti-note"));
}

#[test]
fn iti_is_asked_only_outside_bes_cloud_and_drives_its_own_rule() {
	let shows =
		|e: &pollen_server::ruleset::Evaluation| e.visible_questions.iter().any(|q| q == "iti_use");
	assert!(!shows(&evaluate(
		&v1(),
		&with(sized(), json!({ "hosting_where": "allbes" }))
	)));
	assert!(shows(&evaluate(
		&v1(),
		&with(sized(), json!({ "hosting_where": "allclient" }))
	)));

	// Defaulting to "none" must not assert that an appliance is in play.
	let none = evaluate(
		&v1(),
		&with(sized(), json!({ "hosting_where": "allclient" })),
	);
	assert!(!fired_ids(&none).contains(&"iti-note"));

	let some = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "hosting_where": "allclient", "iti_use": "some" }),
		),
	);
	assert!(fired_ids(&some).contains(&"iti-note"));
}

#[test]
fn an_all_iti_deployment_has_no_os_to_choose() {
	// Iti is a fixed ARM64 appliance, so when every site outside BES cloud runs
	// one there is no operating system or provisioning decision left.
	let all_iti = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "central": "bescloud", "hosting_where": "allclient", "iti_use": "all" }),
		),
	);
	for q in ["platform", "onprem_form"] {
		assert!(
			!all_iti.visible_questions.iter().any(|v| v == q),
			"{q} should be hidden when every site runs an appliance"
		);
	}
	// The client still has a network to configure for those appliances.
	assert!(fired_ids(&all_iti).contains(&"onprem-network"));

	// Some sites on their own servers keeps the questions.
	let some_iti = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "central": "bescloud", "hosting_where": "allclient", "iti_use": "some" }),
		),
	);
	assert!(some_iti.visible_questions.iter().any(|v| v == "platform"));
}

#[test]
fn declining_telemetry_is_an_off_default_opt_out() {
	let off = evaluate(
		&v1(),
		&with(sized(), json!({ "telemetry": "no", "tupaia": "no" })),
	);
	assert!(fired_ids(&off).contains(&"telemetry-off"));
	assert_eq!(off.verdict, Verdict::NonDefault);

	// The outbound allowance only applies when the client hosts servers.
	let on = evaluate(
		&v1(),
		&answers(json!({ "telemetry": "yes", "central": "clienthosted" })),
	);
	let ids = fired_ids(&on);
	assert!(ids.contains(&"telemetry-on"));
	assert!(!ids.contains(&"telemetry-off"));
}

#[test]
fn client_network_items_need_on_prem() {
	let common = |where_: &str| {
		json!({
			"catchment": "c0",
			"facilities": "f0",
			"mobile": "m0",
			"central": "bescloud",
			"hosting_where": where_,
			"remote": "tailscale",
			"timesync": "outbound",
			"telemetry": "yes",
		})
	};
	// All-cloud: the client-side network allowances don't apply.
	let cloud = evaluate(&v1(), &answers(common("allbes")));
	let cloud_ids = fired_ids(&cloud);
	for id in ["remote-tailscale", "time-outbound", "telemetry-on"] {
		assert!(!cloud_ids.contains(&id), "{id} should not fire all-cloud");
	}
	// With client-hosted facilities, they do.
	let onprem = evaluate(&v1(), &answers(common("allclient")));
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
		&answers(json!({ "telemetry": "no", "tupaia": "no", "mobile": "m2" })),
	);
	assert!(fired_ids(&mobile).contains(&"block-telemetry-mobile"));
	assert_eq!(mobile.verdict, Verdict::Blocking);
}

#[test]
fn windows_requires_time_sync_setup() {
	// Windows servers don't get time sync for free the way the Linux servers do,
	// so choosing Windows always raises the requirement to configure it.
	let eval = evaluate(&v1(), &with(sized(), json!({ "platform": "windows" })));
	assert!(fired_ids(&eval).contains(&"time-windows"));
}

#[test]
fn dns_arrangement_targets_the_consequence() {
	// Each arrangement fires its own consequence. Only the BES subdomain, where
	// BES owns the domain and the certificates outright, stays on the default
	// path; the rest each add a cost or a client-side step.
	let subdomain = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "dns": "bes", "dns_arrangement": "bes_subdomain", "telemetry": "yes", "backup_capability": "yes" }),
		),
	);
	assert!(fired_ids(&subdomain).contains(&"dns-bes-subdomain"));

	let client_domain = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "dns": "bes", "dns_arrangement": "client_domain" }),
		),
	);
	let ids = fired_ids(&client_domain);
	assert!(ids.contains(&"dns-bes-client-domain"));
	assert!(!ids.contains(&"dns-bes-subdomain"));
	assert_eq!(client_domain.verdict, Verdict::NonDefault);

	let soa = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "dns": "bes", "dns_arrangement": "client_subdomain" }),
		),
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
			.any(|g| g.at == "backup_capability" && g.message.contains("retention"))
	);
}
