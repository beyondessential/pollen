# Agent rules for pollen

Pollen is the public-facing Tamanu deployment onboarding wizard: a standalone
axum backend that embeds a React + MUI + Vite SPA and owns its own PostgreSQL
database. It is self-contained — no shared code, database, or auth with any
other service. It stores no client names, no free text, and no sensitive data.

Read the specs (the durable "what") and the active plan in
[`docs/plans/`](docs/plans/) before making changes:
- [`.workhorse/specs/wizard/onboarding.md`](.workhorse/specs/wizard/onboarding.md) — the tool and its engine (lifecycle, ruleset binding, outputs).
- [`.workhorse/specs/wizard/ruleset.md`](.workhorse/specs/wizard/ruleset.md) — the v1 question flow and the consequences it encodes.

## Development workflow
- Specs first: update `.workhorse/specs/` per [`.workhorse/rules.md`](.workhorse/rules.md), then implement, then test.
- For large work, write a plan in `docs/plans/<slug>.md`, then implement.
- `just check` for compilation; `just test` for the suite (RAM-backed Postgres).

## Conventions (carried from the canopy stack)
- **Wire types are generated.** `web/src/api-types.ts` ← `web/openapi.json` ←
  the `#[utoipa::path]` annotations. Run `just gen-openapi` after changing any
  handler's request/response shape, then commit both files.
- **Handlers** under `crates/pollen-server/src/fns/<module>.rs` are bare axum
  handlers; each module exposes `routes()` and is mounted under `/api/<module>`.
- **The SPA** is embedded from `web/dist/` at build time; `build.rs` runs the
  Vite build unless skipped for dev. The dev loop is the API binary + Vite proxy.
- **Migrations** live in `migrations/` (one database — no multi-schema).
  Create them with the diesel CLI via `just migration <name>`, never by hand.
- **The ruleset is data**, not code: `ruleset/v1.json`, content-addressed by
  hash. The engine loads and evaluates it.

## Naming & copy
- No seedling/legacy jargon.
- Nothing deployment-specific is hardcoded (public URL, ruleset repo, DB) — all config.

## Version control
- This repo is colocated jj + git; prefer `jj` for local VCS operations.
