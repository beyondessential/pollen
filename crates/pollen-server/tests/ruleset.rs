//! The v1 ruleset parses, validates, hashes deterministically, and evaluates to
//! the expected verdicts/consequences for representative configurations,
//! including the escape hatches a non-technical user relies on: assumed
//! defaults, "I'm not sure" open items, and the questions that still must be
//! answered.

use pollen_server::ruleset::{
	Answers, Audience, Ruleset, Severity, Verdict, evaluate,
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

fn requirement_ids(eval: &pollen_server::ruleset::Evaluation) -> Vec<&str> {
	eval.requirements.iter().map(|r| r.id.as_str()).collect()
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
	// cloud/client-hosted/Iti split, upgrades slower than support covers.
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
			"cadence": "lessoften",
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
		("platform", "linux"),
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
fn the_whole_domain_chain_is_assumed() {
	// BES managing the names is the standard arrangement, so a reader who never
	// opens the technical section still lands on a name under tamanu.app.
	let eval = evaluate(&v1(), &answers(sized()));
	let assumed: Vec<(&str, &str)> = eval
		.assumed
		.iter()
		.map(|a| (a.question.as_str(), a.option.as_str()))
		.collect();
	assert!(assumed.contains(&("dns", "bes")), "got {assumed:?}");
	// The follow-up is only reachable through that assumption, so it has to be
	// resolved in the same settling pass.
	assert!(assumed.contains(&("dns_arrangement", "bes_subdomain")));
	assert!(fired_ids(&eval).contains(&"dns-bes-subdomain"));

	// Declining still works and takes the follow-up out with it.
	let declined = evaluate(&v1(), &with(sized(), json!({ "dns": "unsure" })));
	assert!(declined.open_items.iter().any(|o| o == "dns"));
	assert!(
		!declined
			.visible_questions
			.iter()
			.any(|q| q == "dns_arrangement")
	);
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
fn linux_carries_no_architecture_penalty() {
	// Architecture (ARM64 or AMD64) isn't asked, so Linux is the blessed
	// default with no off-default consequence attached.
	let eval = evaluate(&v1(), &with(sized(), json!({ "platform": "linux" })));
	assert!(fired_ids(&eval).contains(&"plat-image"));
	assert_eq!(eval.verdict, Verdict::Clear);
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

	// Defaulting to "none" must not assert that a mini-server is in play.
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
	// Iti is a fixed ARM64 mini-server, so when every site outside BES cloud
	// runs one there is no operating system or provisioning decision left.
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
			"{q} should be hidden when every site runs a mini-server"
		);
	}
	// The client still has a network to configure for those mini-servers.
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

#[test]
fn short_retention_is_a_recovery_tradeoff() {
	// Keeping a few days covers dashboards and upgrade tests but not recovery,
	// so it has to read as a real choice rather than a neutral preference.
	let short = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "backup_capability": "yes", "retention": "low", "tupaia": "no" }),
		),
	);
	assert!(fired_ids(&short).contains(&"low-retention"));
	assert_eq!(short.verdict, Verdict::NonDefault);

	let full = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "backup_capability": "yes", "retention": "full", "tupaia": "no" }),
		),
	);
	assert!(!fired_ids(&full).contains(&"low-retention"));
}

#[test]
fn upgrading_slower_than_the_support_window_is_flagged() {
	// BES supports the last 10 releases, so "less often" is the only cadence
	// that can put a deployment on an unsupported version.
	for ok in ["release", "twomonths"] {
		let eval = evaluate(&v1(), &with(sized(), json!({ "cadence": ok })));
		assert!(
			!fired_ids(&eval).contains(&"cadence"),
			"{ok} should not raise the support warning"
		);
	}
	let slow = evaluate(&v1(), &with(sized(), json!({ "cadence": "lessoften" })));
	assert!(fired_ids(&slow).contains(&"cadence"));
	assert_eq!(slow.verdict, Verdict::NonDefault);
}

#[test]
fn sydney_is_the_only_named_region() {
	// BES serves Africa, the Middle East and Asia, so a second named region
	// would have to be one of those; anything else is "another AWS region".
	let region = v1().question("region").cloned().expect("region question");
	let ids: Vec<&str> = region.options.iter().map(|o| o.id.as_str()).collect();
	assert_eq!(ids, vec!["sydney", "otheraws"]);
	assert_eq!(region.default.as_deref(), Some("sydney"));
}

#[test]
fn an_off_standard_choice_yields_both_an_action_and_an_acknowledgement() {
	// Windows asks something of the client (licences) and costs them something
	// (slower support). Those are two different things about one choice, so they
	// are two consequences: the work sits with the client's actions, the cost
	// sits with what they are opting into.
	let eval = evaluate(&v1(), &with(sized(), json!({ "platform": "windows" })));
	let ids = fired_ids(&eval);
	assert!(ids.contains(&"plat-windows-licence"), "the action");
	assert!(ids.contains(&"plat-windows"), "the acknowledgement");

	let by = |id: &str| {
		eval.consequences
			.iter()
			.find(|c| c.id == id)
			.map(|c| &c.consequence)
			.expect("fired")
	};
	assert_eq!(by("plat-windows-licence").audience, Audience::Client);
	assert_eq!(by("plat-windows-licence").severity, Severity::Default);
	assert_eq!(by("plat-windows").audience, Audience::Record);
	assert_eq!(by("plat-windows").severity, Severity::NonDefault);
}

#[test]
fn nothing_off_the_standard_path_sits_in_an_actions_group() {
	// The artifact groups by audience: Client and BES hold work to do, and the
	// record holds what is being accepted. An off-standard consequence is an
	// acknowledgement, so it must not land in a list of actions, however it is
	// triggered.
	for rule in &v1().rules {
		if rule.consequence.severity == Severity::Default {
			continue;
		}
		assert_eq!(
			rule.consequence.audience,
			Audience::Record,
			"{} is off the standard path but addressed to an actions group; \
			 split it into an action and an acknowledgement",
			rule.id
		);
	}
}

#[test]
fn pricing_and_sla_drivers_reach_the_pricing_group() {
	// The pricing and partnerships team reads one list rather than the whole
	// record, so anything that moves the price or the support commitment has to
	// raise its own item for them.
	let eval = evaluate(
		&v1(),
		&with(
			sized(),
			json!({
				"integrations": ["other_nonfhir"],
				"platform": "windows",
				"remote": "other",
				"cadence": "lessoften",
				"telemetry": "no",
				"tupaia": "no",
			}),
		),
	);
	let ids = fired_ids(&eval);
	for expected in [
		"price-size",
		"price-integrations",
		"price-nonfhir",
		"price-windows",
		"price-remote",
		"sla-telemetry",
		"sla-cadence",
	] {
		assert!(ids.contains(&expected), "expected {expected}; got {ids:?}");
	}

	// They are work, not acknowledgements, so they stay on the standard path and
	// out of the warnings the client is asked to accept.
	for c in &eval.consequences {
		if c.consequence.audience == Audience::Pricing {
			assert_eq!(
				c.consequence.severity,
				Severity::Default,
				"{} is pricing work, so it must not double as a warning",
				c.id
			);
		}
	}
}

// ── Compute requirements ─────────────────────────────────────────────────────

#[test]
fn compute_requirements_track_the_classes_present() {
	// The default path sizes small, all-client facilities with BES cloud Central.
	// Central is BES-hosted so it carries no client requirement; the facilities
	// and the workstations do; there are no mobile devices.
	let eval = evaluate(&v1(), &answers(sized()));
	let ids = requirement_ids(&eval);
	assert!(ids.contains(&"req-facility"), "got {ids:?}");
	assert!(ids.contains(&"req-workstation"), "got {ids:?}");
	assert!(
		!ids.contains(&"req-central"),
		"BES hosts Central by default"
	);
	assert!(!ids.contains(&"req-mobile"), "no mobile users by default");
	assert!(!ids.contains(&"req-iti"));
}

#[test]
fn an_all_cloud_deployment_only_needs_workstations() {
	// Everything BES-hosted, no mobile: the client provisions nothing but the
	// devices staff use to reach Tamanu.
	let eval = evaluate(
		&v1(),
		&answers(json!({
			"catchment": "c0",
			"facilities": "f0",
			"mobile": "m0",
			"central": "bescloud",
			"hosting_where": "allbes",
		})),
	);
	assert_eq!(requirement_ids(&eval), vec!["req-workstation"]);
}

#[test]
fn a_client_hosted_central_carries_its_own_requirement() {
	let eval = evaluate(&v1(), &with(sized(), json!({ "central": "clienthosted" })));
	assert!(requirement_ids(&eval).contains(&"req-central"));
}

#[test]
fn mobile_users_bring_a_mobile_device_requirement() {
	let with_mobile = evaluate(&v1(), &with(sized(), json!({ "mobile": "m2" })));
	assert!(requirement_ids(&with_mobile).contains(&"req-mobile"));

	// An unsure mobile count asserts nothing, so no device requirement fires.
	let unsure = evaluate(&v1(), &with(sized(), json!({ "mobile": "m_unsure" })));
	assert!(!requirement_ids(&unsure).contains(&"req-mobile"));
}

#[test]
fn iti_replaces_the_facility_server_requirement_when_every_site_runs_one() {
	// Some sites on Iti keeps the facility-server requirement for the rest.
	let some = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "hosting_where": "allclient", "iti_use": "some" }),
		),
	);
	let some_ids = requirement_ids(&some);
	assert!(some_ids.contains(&"req-facility"));
	assert!(some_ids.contains(&"req-iti"));

	// Every site on Iti: there is no client-run facility server left to spec.
	let all = evaluate(
		&v1(),
		&with(
			sized(),
			json!({ "hosting_where": "allclient", "iti_use": "all" }),
		),
	);
	let all_ids = requirement_ids(&all);
	assert!(all_ids.contains(&"req-iti"));
	assert!(!all_ids.contains(&"req-facility"));
}

#[test]
fn a_ruleset_stored_without_requirements_still_loads() {
	// A finalised artifact bound before compute requirements existed has no
	// `requirements` field in its stored JSON. The append-only model must still
	// read it (spec WIZ, the engine's model is append-only).
	let stored = json!({
		"questions": [
			{ "id": "catchment", "kind": "Band", "label": "?", "options": [ { "id": "c0", "label": "?" } ] },
		],
		"rules": [],
	});
	let ruleset: Ruleset = serde_json::from_value(stored).expect("loads without requirements");
	assert!(ruleset.requirements.is_empty());
	let eval = evaluate(&ruleset, &answers(json!({ "catchment": "c0" })));
	assert!(eval.requirements.is_empty());
}

#[test]
fn every_requirement_names_a_class_and_at_least_one_spec_row() {
	// A profile with no rows at any size would render an empty class heading.
	for r in &v1().requirements {
		assert!(!r.class.is_empty(), "requirement {} has no class", r.id);
		assert!(
			!r.specs.is_empty() || !r.by_size.is_empty(),
			"requirement {} has no spec rows",
			r.id
		);
	}
}

#[test]
fn a_sized_server_scales_its_specs_with_the_size_band() {
	// The client-hosted server profiles pick their processor, memory and storage
	// from the derived size band, and lead with those rows.
	let spec = |eval: &pollen_server::ruleset::Evaluation, id: &str, label: &str| -> String {
		eval.requirements
			.iter()
			.find(|r| r.id == id)
			.unwrap_or_else(|| panic!("{id} present"))
			.specs
			.iter()
			.find(|s| s.label == label)
			.unwrap_or_else(|| panic!("{id} has a {label} row"))
			.value
			.clone()
	};

	// Tiny (the lightest band) versus Large, client-hosted throughout.
	let tiny = evaluate(
		&v1(),
		&answers(json!({ "catchment": "c0", "facilities": "f0", "central": "clienthosted" })),
	);
	assert_eq!(
		spec(&tiny, "req-central", "Processor"),
		"2 cores, x86_64 or ARM64"
	);
	assert_eq!(spec(&tiny, "req-central", "Storage"), "480 GB SSD");

	let large = evaluate(
		&v1(),
		&answers(json!({ "catchment": "c3", "facilities": "f0", "central": "clienthosted" })),
	);
	assert_eq!(
		spec(&large, "req-central", "Processor"),
		"8 cores, x86_64 or ARM64"
	);
	assert_eq!(spec(&large, "req-central", "Memory"), "32 GB");
	assert_eq!(spec(&large, "req-central", "Storage"), "2 TB SSD");

	// The size-varying rows lead; the invariant network/OS rows follow.
	let central = large
		.requirements
		.iter()
		.find(|r| r.id == "req-central")
		.unwrap();
	let labels: Vec<&str> = central.specs.iter().map(|s| s.label.as_str()).collect();
	assert_eq!(
		labels,
		vec![
			"Processor",
			"Memory",
			"Storage",
			"Network",
			"Operating system"
		]
	);
}

#[test]
fn the_smallest_band_advises_against_buying_a_server() {
	// A Tiny deployment that still chooses to self-host is steered toward BES
	// hosting or an Iti rather than dedicated hardware.
	let note = |eval: &pollen_server::ruleset::Evaluation| -> String {
		eval.requirements
			.iter()
			.find(|r| r.id == "req-central")
			.unwrap()
			.note
			.clone()
			.unwrap_or_default()
	};

	let tiny = evaluate(
		&v1(),
		&answers(json!({ "catchment": "c0", "facilities": "f0", "central": "clienthosted" })),
	);
	assert!(
		note(&tiny).contains("cost-effective"),
		"tiny central should carry the hosting advisory; got {:?}",
		note(&tiny)
	);

	// A larger band carries no such advisory.
	let large = evaluate(
		&v1(),
		&answers(json!({ "catchment": "c3", "facilities": "f0", "central": "clienthosted" })),
	);
	assert!(!note(&large).contains("cost-effective"));
}

#[test]
fn the_operating_system_row_states_the_platform_chosen() {
	// The requirement reports the reader's own selection rather than listing the
	// options, so exactly one operating system row survives.
	let os = |platform: &str| -> Vec<String> {
		let eval = evaluate(
			&v1(),
			&with(
				sized(),
				json!({ "central": "clienthosted", "platform": platform }),
			),
		);
		eval.requirements
			.iter()
			.find(|r| r.id == "req-central")
			.expect("central present")
			.specs
			.iter()
			.filter(|s| s.label == "Operating system")
			.map(|s| s.value.clone())
			.collect()
	};

	assert_eq!(os("linuxarm"), vec!["Linux on ARM64"]);
	assert_eq!(os("linuxamd"), vec!["Linux on AMD64"]);
	assert_eq!(os("windows"), vec!["Windows Server"]);
}

#[test]
fn mobile_devices_need_android_13() {
	let eval = evaluate(&v1(), &with(sized(), json!({ "mobile": "m2" })));
	let mobile = eval
		.requirements
		.iter()
		.find(|r| r.id == "req-mobile")
		.expect("mobile present");
	let os = mobile
		.specs
		.iter()
		.find(|s| s.label == "Operating system")
		.expect("has an OS row");
	assert_eq!(os.value, "Android 13 or newer");
}

#[test]
fn an_unsized_draft_falls_back_to_the_lightest_band() {
	// A draft where the bands aren't answered yet has no derived size, so a sized
	// server shows the lightest band's rows rather than none.
	let eval = evaluate(&v1(), &answers(json!({ "central": "clienthosted" })));
	assert!(!eval.derived.contains_key("size"));
	let central = eval
		.requirements
		.iter()
		.find(|r| r.id == "req-central")
		.expect("central still shown");
	let processor = central
		.specs
		.iter()
		.find(|s| s.label == "Processor")
		.expect("has a processor row");
	assert_eq!(processor.value, "2 cores, x86_64 or ARM64");
}

#[test]
fn a_standard_plan_still_has_something_to_price() {
	// Even a plan entirely on the blessed path costs something to host, so the
	// pricing group is never empty.
	let eval = evaluate(&v1(), &answers(sized()));
	assert!(
		eval.consequences
			.iter()
			.any(|c| c.consequence.audience == Audience::Pricing),
		"the pricing group should never be empty"
	);
}
