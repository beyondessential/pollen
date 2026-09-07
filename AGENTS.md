<!-- BEGIN:workhorse 0.3.0 -->
# Workhorse framework

This workspace uses [Workhorse](https://github.com/beyondessential/workhorse), a spec-driven development workbench. Workhorse ships skills (invokable prompts) and reference docs into this repo to shape how AI agents work here.

- **Skills** live at `.agents/skills/` — each skill is a folder containing a `SKILL.md` with YAML frontmatter and a prompt body. `.claude/skills/` is a symlink to the same folder so Claude Code picks them up natively
- **Reference docs** live at `.agents/docs/` — long-form guidance that skill bodies cite by path (spec format conventions and similar)
- **Specs** live at `.workhorse/specs/` — acceptance criteria for each piece of work, organised into areas by subdirectory

When picking up a task, read the skill whose folder name matches what you're being asked to do — its `SKILL.md` describes how to approach the work and which reference docs to follow.

Workhorse keeps this section, the skills, and the reference docs current automatically: the first agent turn of a session smart-merges the latest release over your local edits, so your deliberate changes survive. Edit or remove it freely.
<!-- END:workhorse -->

# Agent rules for pollen

Pollen is the public-facing Tamanu deployment onboarding wizard: a standalone
axum backend that embeds a React + MUI + Vite SPA and owns its own PostgreSQL
database. It is self-contained — no shared code, database, or auth with any
other service. It stores no client names, no free text, and no sensitive data.

Read [`.workhorse/specs/wizard/onboarding.md`](.workhorse/specs/wizard/onboarding.md)
(the durable "what" — the tool, engine, lifecycle, ruleset binding, outputs) and
the active plan in [`docs/plans/`](docs/plans/) before making changes. The
concrete v1 questions and consequences live in the ruleset, documented inline
where it is authored — not in a spec.

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
- **The ruleset is data**, not code. It is authored in RON at `ruleset.ron`,
  normalized to canonical JSON, and content-addressed by hash. Versions are git
  branches of this one file, not parallel files. The concrete
  questions/consequences and their rationale are documented inline there. The
  engine loads and evaluates it.

## Naming & copy
- No seedling/legacy jargon.
- Nothing deployment-specific is hardcoded (public URL, ruleset repo, DB) — all config.

## Version control
- This repo is colocated jj + git; prefer `jj` for local VCS operations.
