# Build plan — deployment onboarding wizard (pollen)

Implements [`WIZ`](../../.workhorse/specs/wizard/onboarding.md) (engine, lifecycle,
ruleset binding, outputs). This is the build roadmap (the "how"); the spec is the
durable "what". The concrete v1 questions/consequences are documented inline in
the authored ruleset, not in a spec.

## Shape and stack

A single public-facing service: a Rust axum backend that embeds a React +
MUI + Vite single-page app and talks to its own PostgreSQL database. Mirror
canopy's private-server / private-web conventions (the builder's reference for
the stack), but as a standalone service: its own database, its own image, no
auth.

Nothing operator- or fleet-facing. No client names, no free text, no
sensitive data (see spec §Data and confidentiality).

### Repo layout

```
pollen/
  Cargo.toml                  # workspace
  crates/
    pollen-server/
      build.rs                # embeds web/dist (skippable in dev via env)
      src/
        main.rs               # CLI args, bind, serve
        lib.rs                # router composition
        state.rs              # AppState: db pool, ruleset store, config
        config.rs             # env/CLI config (public base URL, ruleset repo, ...)
        spa.rs                # embedded SPA fallback
        db/                   # diesel models + schema (the ONE wizard DB)
        ruleset/              # format, normalize+hash, evaluate, migrate, resolve
        fns/                  # axum handlers, mounted under /api/<module>
        openapi.rs
        bin/openapi-dump.rs   # dumps openapi.json for TS codegen
        bin/migrate.rs        # runs embedded migrations (prod + e2e)
  web/                        # React + MUI + Vite SPA (pollen-web)
    src/, openapi.json, src/api-types.ts (generated)
  migrations/                 # diesel migrations (single DB, clean — no multi-schema)
  ruleset.ron                 # the canonical ruleset, authored in RON (commented);
                              #   bundled + normalized to canonical JSON + hashed on boot.
                              #   Versions are git branches of this file, not parallel files.
  scripts/ramdisk-pg.sh       # RAM-backed Postgres test harness (port from canopy)
  .github/                    # CI (ci.yml), CD (cd.yml), Dockerfile
  justfile
```

Versions for shared deps (axum, diesel-async, utoipa, jiff, tokio, etc.) are
pinned to the same versions canopy uses — known-good together, not guessed.

## Configuration (nothing hardcoded)

All deployment-specific values come from config (env / CLI), never baked in:

- **Public base URL** — used to build the links the tool hands out (the host
  is expected to be something like `new.tamanu.app`, but is not assumed).
- **Database URL** — its own database, unrelated to canopy's.
- **Ruleset source repository** — the single repository whose refs the
  `?config=<branch>` preview resolves against (see Phase 3). One value;
  fork/arbitrary-URL fetches are rejected by construction.
- **Bind address / port.**

## Phases

Commit incrementally (jj). Each phase is independently reviewable.

### Phase 0 — Scaffold & toolchain
- Cargo workspace + `pollen-server` crate + `web/` Vite app.
- `build.rs` embedding `web/dist` with a dev skip env, mirroring private-server.
- SPA fallback + `/health` + an empty `/api` router + openapi-dump bin.
- Vite dev proxy to the API; the utoipa → `openapi.json` → `openapi-typescript`
  codegen pipeline; `web/src/api.ts` typed-call helper.
- `scripts/ramdisk-pg.sh`, `justfile` (check, test, watch-api, watch-web,
  gen-openapi, migrate, migration, typecheck, build-release), CI/CD + Dockerfile
  (lean image: just the wizard binary + embedded SPA).
- Exit: `just check`, a hello SPA served by the binary, `/health` green.

### Phase 1 — Data model & migrations
- `config_store(config_hash PK, content jsonb, created_at)` — content-addressed
  ruleset store; identical content dedupes to one row.
- `applications(id uuid PK, answers jsonb, config_hash FK→config_store,
  status enum{draft,finalized}, parent_id nullable FK→applications,
  created_at, finalized_at nullable)`.
- Diesel models + the connection pool in `AppState`; `migrate` bin.
- Exit: migrations run on the ramdisk DB; round-trip a row in a db test.

### Phase 2 — Ruleset format & engine
- **Authoring format: RON.** Author the ruleset in RON (`ruleset.ron`), so
  the concrete questions/consequences and their rationale are documented inline
  where they're authored — not in a spec. Parse RON → the typed `Ruleset` model
  → serialize to canonical JSON (recursively key-sorted, compact) → hash. The
  canonical JSON is the machine form stored in `config_store`; comments and
  formatting are dropped by normalization, so they don't affect the hash.
- Define the structure (+ a schema for validation/authoring aid):
  - **questions**: stable `id`, kind (single / multi / band), options
    (stable `id`, label, note), and a visibility condition.
  - **derivations**: e.g. size = highest band among named questions.
  - **rules**: stable `id`, a trigger **condition**, and the tagged consequence
    (severity, types, status, audience, title, detail, optional cost band).
  - A small declarative **condition language** over answers/derived flags:
    `answered`, `equals(q,opt)`, `includes(q,class)`, `all`, `any`, `not`.
    This expresses presence-of-class and cross-field conditions (spec §Triggering).
- The **evaluator**: answers + ruleset → derived flags, the union of triggered
  consequences, and the verdict (blocking > non-default > clear).
- **Normalize + hash**: parse the authoring format, canonicalize to JSON,
  hash → `config_hash`.
- Author the v1 ruleset (analytics intent, integrations, sizing, topology,
  region, platform, on-prem detail, backups capability+retention, cadence,
  networking), incl. the cross-field blocks (analytics↔backups,
  unsupported-platform→blocking). Content modelled on the prototype; values
  documented inline (see content notes below).
- Unit tests: the prototype's demo config and the key blocking combinations
  reproduce the expected verdict + consequence set.
- Exit: engine evaluates the v1 ruleset deterministically; tests green.

### Phase 3 — Ruleset resolution & binding (security-critical)
- On boot: load the bundled `ruleset.ron`, parse + `validate()`, normalize,
  hash, upsert into `config_store`; this hash is the default binding for new
  drafts. Fail fast (panic) if the bundled ruleset is invalid.
- **`?config=<branch>` preview** (spec §Preview against repository refs):
  1. resolve the branch against the configured repo's own refs → commit SHA
     *in that repo*, via the **GitHub REST API** (`reqwest`);
  2. fetch `ruleset.ron` at that SHA, parse + validate + normalize + hash +
     upsert + bind.
  The input is only ever a branch *name*, resolved through the configured repo —
  never a URL. No URL matching, no arbitrary fetch.
  - **Config**: the source repo (`owner/repo`) is one value; an optional token
    lifts the unauthenticated rate limit. The fetched path is fixed (`ruleset.ron`).
  - Abstract the HTTP behind a `RefSource` trait (resolve_ref + fetch_file) so
    the resolver, cache, rate limiter, and rejection paths are unit-tested with a
    fake source — no network in tests.
- **Rate limiting** (the default path is free — it binds the boot-cached hash and
  makes no GitHub call; only previews hit GitHub):
  - A per-branch resolution cache with a short TTL, so repeated previews of the
    same branch reuse the resolved hash without re-hitting GitHub.
  - A global token-bucket limiter on GitHub resolutions; exceeding it returns 429
    (`AppError::RateLimited`). Strong by default (protects GitHub's quota and the
    public, unauthenticated endpoint from abuse).
- **Stable-id migration**: set-diff over question/option ids between two bound
  rulesets → carried / dropped / newly-unanswered, producing the "what changed"
  summary. Unit-tested.
- Exit: preview binds a branch's content hash (via the fake source); the cache
  and rate limiter behave; migration diff is correct.

### Phase 4 — Application lifecycle API
- Endpoints (mounted under `/api`, openapi-annotated):
  - create draft (optional `?config` branch) → returns app id; URL collapses to id.
  - get application (ruleset + answers + evaluation + verdict + lineage).
  - patch answers (draft only).
  - finalize (runs the consistency check; freezes the bound hash; immutable after).
  - update/fork against a new ruleset hash → new draft with lineage; predecessor
    untouched; lands as draft even from a finalized parent.
- `just gen-openapi` regenerates `web/src/api-types.ts`.
- Exit: HTTP tests cover create→patch→finalize→fork and the draft/finalized
  mutability rules (finalized rejects edits).

### Phase 5 — Wizard frontend
- Step flow rendered from the ruleset: question kinds, visibility/hide rules,
  forward-guidance callouts (analytics→backups), derived-size display.
- Live consequences rail + running verdict, updating as answers change.
- Persists answers to the draft; resumable by URL.
- Exit: the flow drives a full draft to finalize against the v1 ruleset.

### Phase 6 — Finalized web view
- Canonical artifact page: by-audience / by-topic toggle, search,
  expand/collapse, section deep-links, and the non-identifying recognition
  header (size, topology shape, region, version, date — no name).
- Exit: a finalized artifact renders both groupings; deep-links work.

### Phase 7 — PDF export
- Print stylesheet over the artifact, sectioned by audience in the spec's order;
  "Download PDF" drives the browser print path. No server-side renderer.
- Exit: print output is correctly sectioned and legible.

### Phase 8 — e2e & polish
- Playwright: a full BES-driven run (load → answer → see consequences →
  finalize → view artifact → print), plus a blocking-combination run and a
  preview/`?config` run. Fixture spawns its own server + Vite against a
  freshly-migrated throwaway DB (port canopy's e2e fixture pattern).
- Visual polish; reduced-motion respect; responsive.

## Deployment (ops-repo dependency, tracked separately)

The image builds in this repo's CD (GHA → ghcr). The K8s workload, public
hostname, Envoy ingress, and CNPG database are an ops/Pulumi change in the ops
repo — the same class as the existing small standalone services. Flag as a
handoff; not built here. Access posture is public/unauthenticated by design.

## Open content values (ruleset data, not engine work)

These live in the authored v1 ruleset, modelled on the prototype's placeholders
and documented inline. They need sign-off from the sales/product side before v1
ships, but don't block the engine:

- Band thresholds for catchment / facilities / mobile clients, and their S/M/L mapping.
- Canonical region options and integration categories/systems.
- Per-choice cost magnitude tiers and ballpark bands.
- The remote-access net-check blurb text.
- Requirement-vs-advisory status of each networking item.

## Notes

- No seedling/legacy jargon in naming or copy.
- Version/verdict indicators always render a state (incl. "unknown"), never hide.
- The default ruleset is bundled and content-hashed; the GitHub-ref preview is
  for previewing unmerged rule changes, and finalize always binds the resolved
  content hash, never a branch name.
