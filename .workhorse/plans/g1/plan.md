# G1 · Display compute requirements in form submission results

## Goal

The finalised artifact and its PDF should present concrete compute requirements
(processor, memory, storage, network, OS/software) for each server and device
class actually present in the deployment. The numbers are the authoritative
recommended base-level specs from the Tamanu "Compute resource recommendations"
reference; which classes appear is driven by the answers.

## Design

Keep the ruleset as data. Add a `requirements` block to the ruleset, evaluated
by the engine exactly like `rules`: each entry has a `when` condition and a
requirement profile (server/device class, a short who-provisions summary, a list
of spec rows, an optional note). The engine emits the union of triggered
requirements; the artifact renders them in a new "Compute requirements" section.

Requirements surface only for classes someone must act on / provision:

- **Central server** — when client-hosted (`central == clienthosted`). BES-cloud
  Central is provisioned by BES, so it carries no client-facing requirement.
- **Facility server** — when client-hosted facilities are present and not every
  site runs an Iti (`hosting_where` allclient/mix AND `iti_use != all`).
- **Tamanu Iti mini-server** — when `iti_use` is some/all. BES-built; only its
  network needs stating.
- **User devices (workstations)** — always. The client provides these regardless
  of hosting.
- **Mobile devices** — when mobile users are in play (`mobile` m1/m2/m3).

Numbers are the recommended base level from the reference doc. Per-size scaling
is deliberately out of scope: the doc gives one base tier and says higher tiers
are advised separately by BES, and inventing per-band figures would state costs
nobody has confirmed. The size band already appears in the artifact header.

## Steps

- [x] Add `Requirement` + `Spec` to the ruleset model; `requirements` on `Ruleset` (serde default)
- [x] Validate requirement id uniqueness in `Ruleset::validate()`
- [x] Emit `requirements` (`TriggeredRequirement`) from the engine `evaluate()`
- [x] Export the new types from `ruleset/mod.rs`
- [x] Author the `requirements` block in `ruleset.ron` with the reference specs
- [x] Regenerate `web/openapi.json` + `api-types.ts`; add wire type re-exports
- [x] Render a "Compute requirements" section in `Artifact.tsx` (+ CSS)
- [x] Update the WIZ spec: outputs, PDF ordering, engine model note
- [x] Rust engine tests for presence gating; extend `tests/ruleset.rs`
- [x] Create `.workhorse/test-cases/g1/overview.md`
- [x] `just check`, `just test`, frontend typecheck
