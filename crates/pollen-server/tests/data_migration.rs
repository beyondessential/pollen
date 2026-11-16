//! The data migration effort model: its scope rules, what each answer adds,
//! how unanswered questions widen the range, and the estimator's endpoints and
//! their gating (spec MIG).

mod common;

use axum::http::StatusCode;
use common::{MIGRATION_PATH, PRICING_PATH, server_without_db};
use pollen_server::data_migration::{
	Area, BUNDLED_DATA_MIGRATION, Combine, ConfidenceLevel, DocLevel, Estimate, EstimateInput,
	MigrationConfig, Narrowing, Span, Tier,
};
use serde_json::{Value, json};

fn config() -> MigrationConfig {
	MigrationConfig::from_ron(BUNDLED_DATA_MIGRATION).expect("bundled migration model")
}

/// The estimate with each unknown at its configured assumption: the arithmetic
/// that works out each end of a range.
fn estimate(input: Value) -> Estimate {
	let input: EstimateInput = serde_json::from_value(input).expect("input parses");
	config().estimate_assuming(&input)
}

/// The estimate as shown: unanswered questions priced at both ends.
fn ranged(input: Value) -> Estimate {
	let input: EstimateInput = serde_json::from_value(input).expect("input parses");
	config().estimate(&input)
}

fn both(low: f64, high: f64) -> Span {
	Span { low, high }
}

fn assert_span(what: &str, got: Span, expected: Span) {
	assert!(
		(got.low - expected.low).abs() < 1e-9 && (got.high - expected.high).abs() < 1e-9,
		"{what}: got {} to {}, expected {} to {}",
		got.low,
		got.high,
		expected.low,
		expected.high
	);
}

fn areas(e: &Estimate) -> Vec<Area> {
	e.areas.iter().map(|a| a.area).collect()
}

fn line(e: &Estimate, area: Area) -> Option<&pollen_server::data_migration::AreaLine> {
	e.areas.iter().find(|a| a.area == area)
}

fn flagged(e: &Estimate, text: &str) -> bool {
	e.flags.iter().any(|f| f.contains(text))
}

fn worked(e: &Estimate, text: &str) -> bool {
	e.workings.iter().any(|w| w == text)
}

fn headline(e: &Estimate) -> Span {
	e.staffing.iter().find(|r| r.as_answered).unwrap().months
}

/// `input` with the contact, code lists and data dictionary all yes, so the
/// documentation multiplies nothing.
fn documented(input: Value) -> Value {
	with(
		&input,
		json!({ "doc_dictionary": "good", "doc_codes": "good", "doc_contact": "good" }),
	)
}

/// `base` with `extra`'s answers laid over it.
fn with(base: &Value, extra: Value) -> Value {
	let mut input = base.clone();
	input
		.as_object_mut()
		.unwrap()
		.extend(extra.as_object().unwrap().clone());
	input
}

/// Lists and staff, one data person, every question at its easy answer: a
/// small total that rounds to the day. Its data person-days are M2's 3 and
/// the first cutover's 4 to 5, with the 10% review allowance: 8 to 9.
fn small() -> Value {
	json!({
		"tier": "reference-and-users", "staffing": "one", "many_users": false,
		"source_kind": "single-system", "source_instances": 1, "source_db": "mysql-or-postgres",
		"doc_dictionary": "good", "doc_codes": "good", "doc_contact": "good",
		"patients_per_day": "small", "review_speed": "fast", "facility_count": 1,
	})
}

/// The reference cases: inputs, and what the estimator gave for each when
/// they were last recorded.
const CASES: &str = concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/tests/fixtures/data_migration_cases.json"
);

/// A range as a reference case records it: `[low, high]`, or null.
fn pair(v: &Value) -> Value {
	if v.is_null() {
		Value::Null
	} else {
		json!([v["low"], v["high"]])
	}
}

/// What a reference case records of a BES-led estimate as shown: every
/// figure, and how many lines of prose.
fn record_bes(e: &Estimate) -> Value {
	let v = serde_json::to_value(e).unwrap();
	let headline = |a: &Value| json!([pair(&a["months"]), pair(&a["person_days"])]);
	let each = |key: &str, f: &dyn Fn(&Value) -> Value| {
		v[key].as_array().unwrap().iter().map(f).collect::<Vec<_>>()
	};
	let count = |key: &str| v[key].as_array().unwrap().len();
	let alternatives = &v["documentation_alternatives"];
	json!({
		"tier": v["tier"],
		"confidence": v["confidence"],
		"documentation": v["documentation"],
		"upper_bound": v["upper_bound"],
		// Each area: code, count, weeks, person-days, basis.
		"areas": each("areas", &|a| json!([
			a["area"], pair(&a["count"]), pair(&a["weeks"]), pair(&a["person_days"]), a["basis"],
		])),
		// Each staffing row: people, as answered, months.
		"staffing": each("staffing", &|r| json!([r["people"], r["as_answered"], pair(&r["months"])])),
		"person_days": pair(&v["person_days"]),
		"scoping_person_days": pair(&v["scoping_person_days"]),
		// Months and person-days if documentation is good, then with no contact.
		"documentation_alternatives": if alternatives.is_null() {
			Value::Null
		} else {
			json!([headline(&alternatives["good"]), headline(&alternatives["no_contact"])])
		},
		// Each question: months and person-days at its cheapest, then costliest.
		"narrowing": each("narrowing", &|n| json!([
			n["question"], headline(&n["cheapest"]), headline(&n["costliest"]),
		])),
		"unknown": v["unknown"],
		"workings": count("workings"),
		"stop_flags": count("stop_flags"),
		"flags": count("flags"),
		"not_included": count("not_included"),
	})
}

/// What a reference case records of a client-prepared estimate.
fn record_import(e: &pollen_server::data_migration::ImportEstimate) -> Value {
	let v = serde_json::to_value(e).unwrap();
	json!({
		// Each program: sites, registry, forms, days, weeks.
		"projects": v["projects"].as_array().unwrap().iter().map(|p| json!([
			p["sites"], p["registry"], p["forms"], pair(&p["days"]), pair(&p["weeks"]),
		])).collect::<Vec<_>>(),
		"templates_days": pair(&v["templates_days"]),
		"total_days": pair(&v["total_days"]),
		"workings": v["workings"].as_array().unwrap().len(),
		"flags": v["flags"].as_array().unwrap().len(),
	})
}

/// Every reference case gives what was recorded for it. With
/// UPDATE_REFERENCE_CASES set, records what each gives instead: for after a
/// deliberate change to the model, whose diff is then reviewed.
#[test]
fn matches_every_reference_case() {
	let mut cases: Value = serde_json::from_str(&std::fs::read_to_string(CASES).unwrap()).unwrap();
	let update = std::env::var_os("UPDATE_REFERENCE_CASES").is_some();
	let config = config();
	let mut failures = Vec::new();
	let mut check = |route: &str, run: &dyn Fn(&Value) -> Value| {
		let list = cases[route].as_array_mut().unwrap();
		assert!(list.len() > 100, "{route}: the cases are missing");
		for case in list {
			let got = run(&case["input"]);
			if update {
				case["expected"] = got;
			} else if case["expected"] != got {
				failures.push(format!(
					"{route} \"{}\"\n  expected {}\n  got      {}",
					case["name"].as_str().unwrap(),
					case["expected"],
					got
				));
			}
		}
	};
	check("bes_led", &|input| {
		record_bes(&config.estimate(&serde_json::from_value(input.clone()).unwrap()))
	});
	check("client_prepared", &|input| {
		record_import(&config.estimate_import(&serde_json::from_value(input.clone()).unwrap()))
	});

	if update {
		// One case a line, so a change to the model diffs case by case.
		let route = |key: &str| {
			cases[key]
				.as_array()
				.unwrap()
				.iter()
				.map(|c| {
					format!(
						"  {{\"name\": {}, \"input\": {}, \"expected\": {}}}",
						c["name"], c["input"], c["expected"]
					)
				})
				.collect::<Vec<_>>()
				.join(",\n")
		};
		let text = format!(
			"{{\n\"source\": {},\n\"bes_led\": [\n{}\n],\n\"client_prepared\": [\n{}\n]\n}}\n",
			cases["source"],
			route("bes_led"),
			route("client_prepared")
		);
		std::fs::write(CASES, text).unwrap();
		return;
	}
	assert!(
		failures.is_empty(),
		"{} reference case(s) differ; if the change is deliberate, record them again with \
		 UPDATE_REFERENCE_CASES=1 and review the diff. The first:\n{}",
		failures.len(),
		failures[..failures.len().min(3)].join("\n")
	);
}

#[test]
fn each_tier_starts_from_its_areas() {
	use Area::*;
	let scope = |tier: &str| areas(&estimate(json!({ "tier": tier })));
	assert_eq!(scope("reference-and-users"), [M0, M1, M2]);
	assert_eq!(scope("patient-register"), [M0, M1, M2, M3]);
	assert_eq!(scope("patient-history"), [M0, M1, M2, M3, M4h]);
	// Clinical forms are optional, like disease or program forms.
	assert_eq!(
		scope("full-ehr"),
		[M0, M1, M2, M3, M4, M5, M7, M8, M9, M11, M12b]
	);
}

#[test]
fn no_tier_starts_from_the_configured_one_and_says_so() {
	let e = estimate(json!({}));
	assert_eq!(e.tier, Some(Tier::PatientHistory));
	assert!(flagged(&e, "Tier not chosen"));
	assert_eq!(e.confidence, ConfidenceLevel::Low);
	assert!(e.unknown.contains(&"tier".to_string()));
}

#[test]
fn the_scope_is_completed_by_what_each_area_depends_on() {
	use Area::*;

	// An encounter-based area brings in encounters; a patient-level one does not.
	let e = estimate(json!({ "tier": "patient-register", "add": ["M7"] }));
	assert_eq!(areas(&e), [M0, M1, M2, M3, M4, M7]);
	assert!(flagged(
		&e,
		"Added encounters (M4): Tamanu records M7 against encounters."
	));
	let e = estimate(json!({ "tier": "patient-register", "add": ["M11f"] }));
	assert_eq!(areas(&e), [M0, M1, M2, M3, M11f]);

	// Anything after encounters raises the history tier's encounters.
	let e = estimate(json!({ "tier": "patient-history", "add": ["M13"] }));
	assert_eq!(areas(&e), [M0, M1, M2, M3, M4, M13]);
	assert!(flagged(
		&e,
		"raised from patient history (M4h) to full encounters (M4)"
	));

	// Encounters bring in patients, and medication administration medications.
	let e = estimate(json!({ "tier": "reference-and-users", "add": ["M6"] }));
	assert_eq!(areas(&e), [M0, M1, M2, M3, M4, M5, M6]);
	assert!(flagged(&e, "MAR (M6) needs medications"));
	assert!(flagged(&e, "Encounters need patients"));

	// The narrower alternative takes the place of the wider one, and the
	// fuller billing the place of opening balances.
	let e = estimate(json!({ "tier": "full-ehr", "add": ["M11f", "M12f"], "form_count": 2 }));
	assert!(areas(&e).contains(&M11f) && !areas(&e).contains(&M11));
	assert!(areas(&e).contains(&M12f) && !areas(&e).contains(&M12b));

	// Scoping and reference data cannot be removed.
	let e = estimate(json!({ "tier": "patient-register", "remove": ["M0", "M1", "M2"] }));
	assert_eq!(areas(&e)[..3], [M0, M1, M2]);
	assert!(flagged(&e, "M0 cannot be removed"));
	assert!(flagged(&e, "M2 cannot be removed"));
}

#[test]
fn forms_follow_their_counts_and_none_is_the_default() {
	use Area::*;

	// A count brings the area in at a week, 3 data person-days, each.
	let e = estimate(documented(
		json!({ "tier": "patient-register", "registry_count": 3 }),
	));
	let registries = line(&e, M10r).expect("disease or program forms in scope");
	assert_eq!(registries.count, Some(both(3.0, 3.0)));
	assert_span("three forms", registries.weeks, both(3.0, 3.0));
	assert_span("three forms", registries.person_days, both(9.0, 9.0));
	// They are patient-level, so encounters stay out.
	assert!(!areas(&e).contains(&M4));
	let e = estimate(documented(json!({ "tier": "full-ehr", "form_count": 2 })));
	assert_span(
		"two clinical forms",
		line(&e, M10f).unwrap().person_days,
		both(6.0, 6.0),
	);

	// No count is none, and is listed as not included; ticked without a
	// count says so. The count is never priced both ways.
	for input in [
		json!({ "tier": "full-ehr" }),
		json!({ "tier": "full-ehr", "add": ["M10f"] }),
		json!({ "tier": "full-ehr", "add": ["M10f"], "form_count": 0 }),
	] {
		let e = ranged(input.clone());
		assert!(line(&e, M10f).is_none(), "{input}");
		assert!(
			e.not_included
				.iter()
				.any(|n| n == "Clinical forms: none counted.")
		);
		assert!(!e.unknown.iter().any(|u| u.contains("form")));
		assert!(!e.narrowing.iter().any(|n| n.question.contains("forms")));
	}
	let e = estimate(json!({ "tier": "full-ehr", "add": ["M10f"] }));
	assert!(flagged(
		&e,
		"Clinical forms ticked with no count: none counted, so left out."
	));
}

#[test]
fn patients_run_from_basic_registration_to_the_full_record() {
	let e = estimate(documented(json!({ "tier": "patient-register" })));
	let m3 = line(&e, Area::M3).expect("patients in scope");
	// Basic registration is 0.5 to 2 weeks; the full record is 5 data
	// person-days more.
	assert_span("patients weeks", m3.weeks, both(0.5, 3.5));
	assert_span("patients person-days", m3.person_days, both(2.0, 11.0));
}

#[test]
fn each_documentation_answer_multiplies_every_area() {
	let base = with(&small(), json!({ "tier": "patient-history" }));
	let doc = |extra: Value| estimate(with(&base, extra));

	// Yes multiplies by one; Partly and No by their own factors, set-up
	// included, and the factors compound.
	let good = doc(json!({}));
	assert!(!good.workings.iter().any(|w| w.contains("every area")));
	for (answer, working) in [
		(
			json!({ "doc_contact": "partial" }),
			"Someone partly knows how the data is stored: every area x1.05.",
		),
		(
			json!({ "doc_contact": "none" }),
			"Nobody knows how the data is stored: every area x1.1.",
		),
		(
			json!({ "doc_codes": "partial" }),
			"Code lists can partly be exported: every area x1.025.",
		),
		(
			json!({ "doc_codes": "none" }),
			"Code lists cannot be exported: every area x1.05.",
		),
		(
			json!({ "doc_dictionary": "partial", "doc_prior_exports": "good" }),
			"Data dictionary partly there or out of date: every area x1.1.",
		),
		(
			json!({ "doc_dictionary": "none", "doc_prior_exports": "good" }),
			"No data dictionary: every area x1.2.",
		),
	] {
		let e = doc(answer.clone());
		assert!(worked(&e, working), "{answer}: {:?}", e.workings);
		for (x, y) in e.areas.iter().zip(&good.areas) {
			assert!(x.weeks.high >= y.weeks.high, "{answer} {:?}", x.area);
		}
		assert!(e.person_days.high >= good.person_days.high, "{answer}");
	}

	// The factor applies to every area's own range: doubled, set-up,
	// reference data and the visit history double.
	let mut doubled = config();
	doubled.documentation.contact.none = 2.0;
	let parse = |v: Value| -> EstimateInput { serde_json::from_value(v).unwrap() };
	let x = doubled.estimate_assuming(&parse(with(&base, json!({ "doc_contact": "none" }))));
	for area in [Area::M0, Area::M1, Area::M4h] {
		assert_span(
			&format!("{area:?} doubled"),
			line(&x, area).unwrap().weeks,
			line(&good, area).unwrap().weeks.map(|w| w * 2.0),
		);
	}
	// Two answers compound.
	let both_none = doc(json!({ "doc_contact": "none", "doc_codes": "none" }));
	assert!(worked(
		&both_none,
		"Nobody knows how the data is stored: every area x1.1."
	));
	assert!(worked(
		&both_none,
		"Code lists cannot be exported: every area x1.05."
	));

	// Earlier exports count only when the data dictionary is not there.
	let exports = "No earlier exports with their queries: every area x1.05.";
	assert!(worked(
		&doc(json!({ "doc_dictionary": "partial", "doc_prior_exports": "none" })),
		exports
	));
	assert!(worked(
		&doc(json!({ "doc_dictionary": "none", "doc_prior_exports": "none" })),
		exports
	));
	assert!(!worked(
		&doc(json!({ "doc_dictionary": "good", "doc_prior_exports": "none" })),
		exports
	));
	let open = ranged(with(&base, json!({ "doc_prior_exports": null })));
	assert!(
		!open
			.narrowing
			.iter()
			.any(|n| n.question.starts_with("Earlier exports")),
		"not asked with a dictionary"
	);

	// Unknown answers are taken as partly, and flagged.
	let unknown = doc(json!({ "doc_dictionary": null }));
	assert!(flagged(
		&unknown,
		"Data dictionary unknown: treated as Partly."
	));

	// Reported as good only when the contact, code lists and dictionary are
	// all yes; none without a contact.
	assert_eq!(good.documentation, DocLevel::Good);
	assert!(good.documentation_alternatives.is_none());
	let none = doc(json!({ "doc_contact": "none" }));
	assert_eq!(none.documentation, DocLevel::None);
	assert_eq!(none.confidence, ConfidenceLevel::Low);
	assert!(none.documentation_alternatives.is_none());

	// Partial documentation shows what each alternative would give: the same
	// figures as answering that way.
	let partial = doc(json!({ "doc_dictionary": "partial", "doc_prior_exports": "none" }));
	assert_eq!(partial.documentation, DocLevel::Partial);
	let alt = partial
		.documentation_alternatives
		.expect("alternatives for partial");
	let all_good = doc(json!({ "doc_prior_exports": "good" }));
	assert_span("good months", alt.good.months, headline(&all_good));
	assert_span(
		"good person-days",
		alt.good.person_days,
		all_good.person_days,
	);
	let no_contact = doc(json!({
		"doc_dictionary": "partial", "doc_prior_exports": "none", "doc_contact": "none",
	}));
	assert_span(
		"no contact months",
		alt.no_contact.months,
		headline(&no_contact),
	);
	assert_span(
		"no contact person-days",
		alt.no_contact.person_days,
		no_contact.person_days,
	);
}

#[test]
fn independent_areas_add_up_narrower_than_stacked() {
	let input: EstimateInput = serde_json::from_value(json!({
		"tier": "full-ehr", "form_count": 4, "facility_count": 2, "source_instances": 2,
		"doc_dictionary": "partial", "doc_codes": "partial", "doc_contact": "partial",
		"doc_prior_exports": "none", "source_kind": "single-system", "source_db": "mysql-or-postgres",
		"patients_per_day": "medium", "history_years": "medium",
	}))
	.unwrap();
	let independent = config();
	let mut stacked = config();
	stacked.combine = Combine::Stacked;
	let (a, b) = (
		independent.estimate_assuming(&input),
		stacked.estimate_assuming(&input),
	);

	// The same areas, each with the same range; only the totals differ.
	for (x, y) in a.areas.iter().zip(&b.areas) {
		assert_span(&format!("{:?}", x.area), x.weeks, y.weeks);
	}
	// The independent total sits inside the stacked one, around the same middle.
	for (what, i, s) in [
		("months", headline(&a), headline(&b)),
		("person-days", a.person_days, b.person_days),
	] {
		assert!(
			i.low >= s.low && i.high <= s.high,
			"{what}: {i:?} inside {s:?}"
		);
		assert!(i.high - i.low < s.high - s.low, "{what}: narrower");
	}
}

#[test]
fn stop_flags_are_raised_without_stopping_the_estimate() {
	let e = estimate(json!({
		"tier": "patient-register", "tamanu_live": true,
		"doc_contact": "none", "doc_dictionary": "none",
	}));
	assert_eq!(e.stop_flags.len(), 2);
	assert!(e.stop_flags[0].contains("already in use"));
	assert!(e.stop_flags[1].contains("no data dictionary"));
	assert!(!e.areas.is_empty());

	let e = estimate(json!({ "tier": "patient-register", "tamanu_live": false }));
	assert!(e.stop_flags.is_empty());
}

#[test]
fn every_estimate_lists_what_it_leaves_out_and_names_its_model() {
	let config = config();
	let e = estimate(json!({ "tier": "reference-and-users" }));
	assert_eq!(e.model_version, config.version);
	assert!(
		e.narrows_after
			.contains("data quality and shape vary widely")
	);
	assert!(
		ranged(json!({}))
			.narrows_after
			.contains("after the initial analysis of the data (M0)")
	);
	let left_out = |e: &Estimate, text: &str| e.not_included.iter().any(|n| n.contains(text));
	assert!(left_out(&e, "Post go-live support (4 to 13"));
	assert!(left_out(&e, "FHIR"));
	assert!(left_out(
		&e,
		"Finding and merging duplicate patient records."
	));
	assert!(left_out(&e, "Clinical forms: none counted."));
	assert!(left_out(&e, "Disease or program forms: none counted."));
	assert!(!left_out(&e, "Building Tamanu forms"));

	// Forms in scope may need building in Tamanu, and are no longer left out.
	let e = estimate(json!({ "tier": "full-ehr", "form_count": 2 }));
	assert!(left_out(&e, "Building Tamanu forms"));
	assert!(!left_out(&e, "Clinical forms: none counted."));
	assert!(left_out(&e, "Disease or program forms: none counted."));
}

#[test]
fn more_people_shorten_the_shared_areas_and_add_alignment() {
	let input =
		|staffing: &str| json!({ "tier": "full-ehr", "form_count": 4, "staffing": staffing });
	let (one, two, three) = (
		estimate(input("one")),
		estimate(input("two")),
		estimate(input("three")),
	);
	// Person-days grow 5% for each data person beyond the first, compounding.
	assert!(!one.workings.iter().any(|w| w.contains("aligned")));
	assert!(worked(
		&two,
		"2 data people: person-days x1.05 for keeping the work aligned."
	));
	assert!(worked(
		&three,
		"3 data people: person-days x1.1025 for keeping the work aligned."
	));
	assert!(one.person_days.high < two.person_days.high);
	assert!(two.person_days.high < three.person_days.high);

	// Only the areas after encounters are shared, so the timeline shortens.
	let months = |e: &Estimate, people: u32| {
		e.staffing
			.iter()
			.find(|r| r.people == Some(people))
			.unwrap()
			.months
	};
	assert!(months(&two, 3).high < months(&two, 2).high);
	assert!(months(&two, 2).high < months(&two, 1).high);
	let answered = |e: &Estimate, people: u32| {
		e.staffing
			.iter()
			.find(|r| r.people == Some(people))
			.unwrap()
			.as_answered
	};
	assert!(answered(&two, 2) && answered(&three, 3));

	// Nothing after encounters: staffing changes nothing on the timeline.
	let e = estimate(json!({ "tier": "patient-history" }));
	assert_eq!(e.staffing.len(), 1);
	assert_eq!(e.staffing[0].people, None);
}

#[test]
fn the_source_adds_its_own_time() {
	let base = small();
	let run = |extra: Value| estimate(with(&base, extra));
	let plain = run(json!({}));

	// Another database adds 2 weeks and 6 person-days; set-up keeps its range.
	let other = run(json!({ "source_db": "other" }));
	assert!(worked(
		&other,
		"Source database not MySQL or Postgres: added 2 weeks and 6 person-days."
	));
	assert_span(
		"set-up",
		line(&other, Area::M0).unwrap().weeks,
		line(&plain, Area::M0).unwrap().weeks,
	);
	assert_span("other database", other.person_days, both(14.0, 15.0));

	// Several systems or a mix add 2 to 4 weeks and 8 to 12 person-days;
	// spreadsheets alone add nothing, but are an upper bound.
	let several = "Several systems or a mix: added 2 to 4 weeks and 8 to 12 person-days.";
	for kind in ["several-systems", "mix"] {
		assert!(worked(&run(json!({ "source_kind": kind })), several));
	}
	let sheets = run(json!({ "source_kind": "spreadsheets" }));
	assert!(!worked(&sheets, several));
	assert!(sheets.upper_bound);
	assert_span("spreadsheets", sheets.person_days, plain.person_days);

	// Separate installs: one adds nothing, 2 to 3 a week, 4 or more two.
	let installs = |n: u32| run(json!({ "source_instances": n }));
	assert_span("one install", installs(1).person_days, plain.person_days);
	for n in [2, 3] {
		assert!(worked(
			&installs(n),
			"2 to 3 separate installs: added 1 week and 3 person-days."
		));
	}
	for n in [4, 9] {
		assert!(worked(
			&installs(n),
			"4 or more separate installs: added 2 weeks and 6 person-days."
		));
	}
}

#[test]
fn sites_tamanu_in_use_and_billing_take_their_configured_time() {
	let base = small();
	let run = |extra: Value| estimate(with(&base, extra));
	let plain = run(json!({}));

	// Each site beyond the first adds 2 weeks and 6 person-days; not
	// answered, there is one.
	let three = run(json!({ "facility_count": 3 }));
	assert!(worked(
		&three,
		"2 more sites going live: added 4 weeks and 12 person-days."
	));
	let blank = ranged(with(&base, json!({ "facility_count": null })));
	assert_span("one site", blank.person_days, plain.person_days);
	assert!(flagged(&blank, "Number of sites not answered: taken as 1"));

	// Tamanu in use adds a week and 3 person-days; not answered, it is not.
	let live = run(json!({ "tamanu_live": true }));
	assert!(worked(
		&live,
		"Tamanu already in use: checking for conflicts with records entered since go-live: added 1 week and 3 person-days."
	));
	assert_span("tamanu in use", live.person_days, both(11.0, 12.0));
	let blank = ranged(with(&base, json!({ "tamanu_live": null })));
	assert_span("tamanu not answered", blank.person_days, plain.person_days);
	assert!(blank.narrowing.is_empty());

	// Billing: 3 to 5 data person-days for unpaid balances, 5 to 10 for the
	// full history.
	let e = run(json!({ "add": ["M12b"] }));
	assert_span(
		"unpaid balances",
		line(&e, Area::M12b).unwrap().person_days,
		both(3.0, 5.0),
	);
	let e = run(json!({ "add": ["M12f"] }));
	assert_span(
		"full history",
		line(&e, Area::M12f).unwrap().person_days,
		both(5.0, 10.0),
	);
}

#[test]
fn size_multiplies_by_patients_a_day_then_years_of_history() {
	let size = |input: Value| {
		estimate(input)
			.workings
			.into_iter()
			.find(|w| w.starts_with("Size: "))
	};
	assert_eq!(
		size(json!({ "tier": "patient-history", "patients_per_day": "medium", "history_years": "large" })),
		Some(
			"Size: x1.1 for 100 to 300 patients a day then x1.5 for over 5 years of history, so x1.65 on M1 and M3 onwards."
				.into()
		)
	);
	assert_eq!(
		size(
			json!({ "tier": "patient-history", "patients_per_day": "small", "history_years": "medium" })
		),
		Some("Size: x1.2 for 2 to 5 years of history, so x1.2 on M1 and M3 onwards.".into())
	);
	assert_eq!(
		size(
			json!({ "tier": "patient-history", "patients_per_day": "small", "history_years": "small" })
		),
		None
	);

	// The multiplier applies to M1 and from M3 on, not to set-up or users.
	let e = estimate(documented(json!({
		"tier": "patient-history", "patients_per_day": "large", "history_years": "large",
		"many_users": false,
	})));
	// 1.25 x 1.5 = 1.875: M1's 3 to 5.5 weeks become 5.6 to 10.3.
	assert_span("M1", line(&e, Area::M1).unwrap().weeks, both(5.5, 10.5));
	assert_span("M0", line(&e, Area::M0).unwrap().weeks, both(1.0, 8.0));
	assert_span("M2", line(&e, Area::M2).unwrap().weeks, both(1.0, 1.0));

	// Unknown years are flagged and add nothing.
	let e = estimate(json!({ "tier": "patient-history", "patients_per_day": "large" }));
	assert!(flagged(&e, "Years of history unknown"));
	assert!(e.unknown.iter().any(|u| u == "years of history"));
	// Without visits in scope, the years are not asked and do not count.
	let e = estimate(
		json!({ "tier": "patient-register", "patients_per_day": "large", "history_years": "large" }),
	);
	assert!(!flagged(&e, "Years of history unknown"));
	assert!(!e.unknown.iter().any(|u| u == "years of history"));
	assert!(worked(
		&e,
		"Size: x1.25 for over 300 patients a day, so x1.25 on M1 and M3 onwards."
	));
}

#[test]
fn staff_to_be_migrated_set_users() {
	let m2 = |many: Value| {
		let e = estimate(documented(
			json!({ "tier": "reference-and-users", "many_users": many }),
		));
		line(&e, Area::M2).unwrap().clone()
	};
	let over = m2(json!(true));
	assert_span("over 500", over.weeks, both(2.0, 2.0));
	assert_span("over 500", over.person_days, both(6.0, 6.0));
	let up_to = m2(json!(false));
	assert_span("up to 500", up_to.weeks, both(1.0, 1.0));
	assert_span("up to 500", up_to.person_days, both(3.0, 3.0));
	assert_span("not answered", m2(Value::Null).weeks, both(1.0, 2.0));
}

#[test]
fn slow_reviews_lengthen_the_worst_case_only() {
	let base = with(
		&small(),
		json!({ "tier": "full-ehr", "form_count": 2, "staffing": "two" }),
	);
	let fast = estimate(base.clone());
	let slow = estimate(with(&base, json!({ "review_speed": "slow" })));
	assert!(worked(
		&slow,
		"Client slow to review: added 8 weeks to the high end of the timeline."
	));
	assert_eq!(headline(&slow).low, headline(&fast).low);
	assert!(headline(&slow).high >= headline(&fast).high + 1.5);
	assert_span("person-days", slow.person_days, fast.person_days);
}

/// The answers that take each unanswered question at its cheapest.
const CHEAPEST: &str = r#"{
	"tier": "reference-and-users",
	"source_kind": "single-system", "source_instances": 1, "source_db": "mysql-or-postgres",
	"doc_dictionary": "good", "doc_codes": "good", "doc_contact": "good",
	"patients_per_day": "small", "history_years": "small", "many_users": false,
	"review_speed": "fast"
}"#;

/// The answers that take each unanswered question at its costliest. Forms,
/// sites and Tamanu in use keep their defaults: none, one, and no.
const COSTLIEST: &str = r#"{
	"tier": "full-ehr",
	"add": ["M0", "M1", "M2", "M3", "M4", "M5", "M6", "M7", "M8", "M9", "M11", "M12f", "M13", "M14"],
	"source_kind": "mix", "source_instances": 4, "source_db": "other",
	"doc_dictionary": "none", "doc_codes": "none", "doc_contact": "none", "doc_prior_exports": "none",
	"patients_per_day": "large", "history_years": "large", "many_users": true,
	"review_speed": "slow"
}"#;

#[test]
fn unanswered_questions_widen_the_range_and_answers_narrow_it() {
	let cheapest: Value = serde_json::from_str(CHEAPEST).unwrap();
	let costliest: Value = serde_json::from_str(COSTLIEST).unwrap();

	// Every question answered: the range is the estimate as answered.
	for answers in [&cheapest, &costliest] {
		let (shown, given) = (ranged(answers.clone()), estimate(answers.clone()));
		assert_span("answered months", headline(&shown), headline(&given));
		assert_span("answered person-days", shown.person_days, given.person_days);
		assert!(shown.narrowing.is_empty());
		assert!(!shown.workings.iter().any(|w| w.starts_with("Not answered")));
	}

	// Nothing answered: from the cheapest answers' low end to the costliest
	// answers' high end.
	let none = ranged(json!({}));
	let (low, high) = (estimate(cheapest), estimate(costliest));
	assert_span(
		"nothing answered",
		headline(&none),
		both(headline(&low).low, headline(&high).high),
	);
	assert_eq!(none.person_days.high, high.person_days.high);
	assert_eq!(none.tier, None);
	assert!(none.workings[0].starts_with("Not answered"));
	// A flag raised at one end only says which.
	assert!(
		none.flags
			.iter()
			.any(|f| f.starts_with("High end: Several systems"))
	);
	// An area in scope at the high end only starts from nothing.
	let tasks = line(&none, Area::M14).unwrap();
	assert_eq!(tasks.weeks.low, 0.0);
	assert!(tasks.weeks.high > 0.0);
	// Questions with a default are never priced both ways.
	for q in &none.narrowing {
		assert!(
			!["Sites going live", "Tamanu already in use there"].contains(&q.question.as_str())
				&& !q.question.contains("forms"),
			"{}",
			q.question
		);
	}

	// The tier moves it most; answering it narrows the range.
	assert_eq!(none.narrowing[0].question, "Closest to what they need");
	let tier = ranged(json!({ "tier": "patient-history" }));
	assert!(headline(&tier).low >= headline(&none).low);
	assert!(headline(&tier).high < headline(&none).high);
	assert!(tier.narrowing.len() < none.narrowing.len());
	for pair in tier.narrowing.windows(2) {
		let swing = |n: &Narrowing| {
			(n.costliest.months.low - n.cheapest.months.low)
				+ (n.costliest.months.high - n.cheapest.months.high)
		};
		assert!(swing(&pair[0]) >= swing(&pair[1]));
	}
}

#[test]
fn client_prepared_programs_add_up_their_forms_registry_sites_and_cases() {
	let config = config();
	let run = |input: Value| config.estimate_import(&serde_json::from_value(input).unwrap());
	let both = |low: f64, high: f64| Span { low, high };

	// A registry, 2 forms, 3 sites, 10,000 to 100,000 cases: 2x(2 to 3) forms
	// + 1 registry + 2x(0.5 to 2) sites + (1 to 2) contingency + 2 buffer.
	let e = run(json!({
		"templates_exist": true,
		"projects": [{ "sites": 3, "registry": true, "forms": 2, "cases": "medium" }],
	}));
	assert_span("worked", e.projects[0].days, both(9.0, 15.0));
	assert_span("weeks", e.projects[0].weeks, both(3.0, 5.0));
	assert_span("total", e.total_days, both(9.0, 15.0));
	assert_eq!(e.templates_days, None);
	assert!(e.flags.is_empty(), "{:?}", e.flags);

	// Without a registry: 2x(1 to 2) forms + 2x(0.5 to 2) sites + (1 to 2) + 2.
	let e = run(json!({
		"templates_exist": true,
		"projects": [{ "sites": 3, "registry": false, "forms": 2, "cases": "small" }],
	}));
	assert_span("no registry", e.projects[0].days, both(6.0, 12.0));

	// Each band, for each form: a registry and one form at one site come to
	// band + 1 + (1 to 2) + 2.
	for (band, low, high) in [
		("small", 1.0, 2.0),
		("medium", 2.0, 3.0),
		("large", 3.0, 5.0),
	] {
		let e = run(json!({
			"templates_exist": true,
			"projects": [{ "sites": 1, "registry": true, "forms": 1, "cases": band }],
		}));
		assert_span(band, e.projects[0].days, both(4.0 + low, 5.0 + high));
	}

	// Unknowns are priced at both ends and flagged. Low: the bottom band, no
	// registry, one site, templates existing: 1 + 1 + 2. High: the top band, a
	// registry, 5 sites, templates built: 5 + 1 + 4x2 + 2 + 2, and 1.
	let e = run(json!({ "projects": [{ "forms": 1 }] }));
	assert_span("unknown", e.projects[0].days, both(4.0, 18.0));
	assert_span("unknown total", e.total_days, both(4.0, 19.0));
	assert_eq!(e.templates_days, Some(both(0.0, 1.0)));
	assert_eq!(e.projects[0].sites, None);
	assert_eq!(e.projects[0].registry, None);
	assert_eq!(e.flags.len(), 4, "{:?}", e.flags);

	// A program with neither a registry nor forms is flagged.
	let e = run(json!({
		"templates_exist": true,
		"projects": [{ "sites": 1, "registry": false, "forms": 0, "cases": "small" }],
	}));
	assert_eq!(e.flags.len(), 1, "{:?}", e.flags);
	assert_span("empty", e.projects[0].days, both(3.0, 4.0));
}

#[tokio::test(flavor = "multi_thread")]
async fn endpoints_hide_behind_the_migration_path() {
	// The estimator stores nothing, so it is served without a database.
	let server = server_without_db();
	for fn_ in ["model", "estimate", "import"] {
		let path = format!("/api/data-migration/{fn_}");
		server
			.post(&path)
			.json(&json!({}))
			.await
			.assert_status(StatusCode::NOT_FOUND);
		server
			.post(&path)
			.add_header("x-migration-path", format!("{}x", MIGRATION_PATH.as_str()))
			.json(&json!({}))
			.await
			.assert_status(StatusCode::NOT_FOUND);
		// The hosting cost edition's path opens nothing here, under either header.
		server
			.post(&path)
			.add_header("x-migration-path", PRICING_PATH.as_str())
			.json(&json!({}))
			.await
			.assert_status(StatusCode::NOT_FOUND);
		server
			.post(&path)
			.add_header("x-pricing-path", PRICING_PATH.as_str())
			.json(&json!({}))
			.await
			.assert_status(StatusCode::NOT_FOUND);
	}
	// Nor does the estimator's path open the hosting cost endpoints.
	server
		.post("/api/pricing/model")
		.add_header("x-pricing-path", MIGRATION_PATH.as_str())
		.json(&json!({}))
		.await
		.assert_status(StatusCode::NOT_FOUND);

	let model: Value = server
		.post("/api/data-migration/model")
		.add_header("x-migration-path", MIGRATION_PATH.as_str())
		.json(&json!({}))
		.await
		.json();
	assert_eq!(model["tiers"].as_array().unwrap().len(), 4);
	assert_eq!(model["areas"].as_array().unwrap().len(), 19);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_estimator_answers_both_routes() {
	// The estimator stores nothing, so it is served without a database.
	let server = server_without_db();
	let got: Value = server
		.post("/api/data-migration/estimate")
		.add_header("x-migration-path", MIGRATION_PATH.as_str())
		.json(&json!({ "tier": "full-ehr", "form_count": 4, "staffing": "two" }))
		.await
		.json();
	assert_eq!(got["tier"], "full-ehr");
	assert_eq!(got["areas"].as_array().unwrap().len(), 12);
	assert_eq!(got["staffing"].as_array().unwrap().len(), 3);
	assert!(got["person_days"]["low"].as_f64().unwrap() > 0.0);

	let got: Value = server
		.post("/api/data-migration/import")
		.add_header("x-migration-path", MIGRATION_PATH.as_str())
		.json(&json!({
			"templates_exist": true,
			"projects": [{ "sites": 1, "registry": true, "forms": 1, "cases": "small" }],
		}))
		.await
		.json();
	assert_eq!(got["total_days"], json!({ "low": 5.0, "high": 7.0 }));
}
