# Internal at-cost pricing calculator

Port the Tamanu hosting cost model into pollen as an internal edition: a calculator and a priced copy of the wizard, sharing one model computed on the server.

## Decisions (Edwin, 3 Nov 2026)

- **Gating:** an unguessable path segment, set by `PRICING_PATH` (at least 16 URL-safe characters). The SPA serves the edition at `/<path>/` (the calculator), `/<path>/new` and `/<path>/a/<id>`. The pricing endpoints take the same segment in an `x-pricing-path` header and answer 404 otherwise. With it unset, the edition is off.
- **Cloud facility servers:** the wizard guesses (facilities in the band) × (share in BES cloud), with both tables in `pricing.ron` and tagged Assumed. A slider appears only for a mix, in place of the balance question in the wizard and under "A mix" in the calculator.
- **Calculator simplified (5 Nov):** no Windows controls, no always-on clone (one "Upgrade smoke testing on a clone" yes/no at the default uptime), Tupaia reporting on by default, and a size estimator whose band comes from the ruleset's own derivation via `pricing/size`. The model and API still price Windows and always-on.
- **Backups and clone are independent.** Turning backups off drops only the backups share, so backups never look more expensive than they are.
- **Large keeps the Slab box** (m6g.2xlarge, 1,500 GB). It's editable in the band table, where a fifth band can also go.

## Design notes

- The Python reference model wasn't used. The Rust port matches every case in `test_cases.json` (copied in as `tests/fixtures/pricing_cases.json`) to the cent.
- All the numbers live in `pricing.ron` at the repo root, bundled like the ruleset and parsed at boot, so a bad edit fails fast. There's no production-branch polling for pricing: changing a figure means a redeploy.
- How wizard answers map to inputs is configuration too, written in the ruleset's condition language against effective answers (assumed answers filled in, hidden questions' answers dropped). No ruleset question id appears in the Rust code.
- Margin and FX are config fields applied only to the displayed total, at 0% and USD for now. The label gains "incl. N% margin" once the margin is above zero.
- The priced figures are hidden in print, so a PDF made from the internal edition carries none.

## Checklist

- [x] Spec: Internal hosting cost (COST), with a cross-reference from the wizard spec
- [x] `pricing.ron` with tagged parameters, the band table and the wizard mapping
- [x] Cost model, fixture test (297 cases), unit tests
- [x] Pricing endpoints (`model`, `cost`, `plan`) behind the path header; OpenAPI and types regenerated
- [x] Edition routing in the SPA; plan links keep the prefix
- [x] Calculator page; estimates are tagged on the breakdown lines rather than listed in a panel
- [x] Running cost in the wizard rail, and above a finalised plan
- [x] e2e for the calculator, the priced wizard and the gating
- [ ] Set `PRICING_PATH` on the deployment and share the URL internally
