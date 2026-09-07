# Simplify the estimate questionnaire for non-technical users

Making the wizard answerable by someone who doesn't know every answer, and
letting them finalise an interim artifact that BES completes later.

## Design decisions

**Two escape hatches, not one.** They mean different things and are authored
per question in the ruleset:

- *"I'm not sure"* — a real option the user picks. Nothing is assumed; the
  question is recorded as an open item on the artifact. Authored as an option
  carrying `unsure: true`.
- *Assumed default* — the blessed-path answer applied when the question is left
  blank, recorded as an assumption the client can correct. Authored as
  `default: Some("<option id>")` on the question.

The test for which a question gets: is the blessed path overwhelmingly what
happens, and does getting it wrong barely move the estimate? The three sizing
questions have no blessed default at all, which is why they are the minimum set.

**Defaults are applied to a fixed point.** Applying a default can reveal a
question that itself has a default, so the engine iterates visibility and
default application until stable rather than doing a single pass.

**Unknowns suppress what they gate.** An `unsure` answer matches no condition,
so questions gated on it stay hidden without special handling. They surface as
open items instead.

**Interim artifacts sit between "normal artifact with a gaps section" and a
distinct lifecycle state.** No new database status and no migration: an artifact
is interim when its evaluation carries open items. That derived flag drives a
badge, a provisional verdict, and an open-items section. Completing one is a
fork, the same as any other change.

## Question flow

Four sections, authored as data so the ruleset keeps owning the flow:

1. **Essentials** — catchment, facilities, mobile clients. The minimum set.
2. **Your deployment** — Tupaia (sold as "dashboards and reporting"), integrations.
3. **Hosting** — the hosting mix and where central runs.
4. **Advanced** — collapsed by default, every technical question, all defaulted.

## Simplifications to the ruleset

- Customer cloud and true on-prem were used identically in every rule, so they
  collapse into one "client hosted" option.
- Bare metal vs virtualised *does* drive distinct provisioning requirements, so
  that distinction moves to an advanced follow-up rather than being lost.
- The facility mix becomes a rough percentage split across BES cloud, client
  hosted, and Tamanu Iti, with the benefits of each stated inline.

## Steering toward the blessed path

Non-blessed choices carry an inline warning shown at the point of selection
(`warn` on the option), on top of the consequence they already push into the
ledger. Linux on ARM64, the tamanu.app domain, and Tailscale are the paths
being steered toward.

## Build steps

- [x] Model: `default` + `section` on Question, `unsure` + `warn` on Opt, `Mix` question kind, `sections` on Ruleset
- [x] Answers: `Mix` variant, share accessor
- [x] Condition: `HasShare`
- [x] Engine: fixed-point defaults, `assumed` / `open_items` / `effective` on the evaluation
- [x] Ruleset: reorder into sections, relabel, collapse hosting options, add the mix, add warnings
- [x] Handler: allow finalise with open items, expose the new question fields
- [x] Regenerate OpenAPI types
- [x] Wizard: sections with a collapsed advanced group, mix control, unsure choices, inline warnings
- [x] Artifact: interim badge, open items, assumptions
- [x] Build, test, run

## Environment notes

Two things this machine needed before the project would build and run:

- The pinned `stable` Rust toolchain was 1.85.1 while dependencies require 1.88+,
  so `rustup update stable` was run (now 1.98.1).
- Postgres was not installed. Something else already occupies 5432 and wants
  SCRAM credentials, so `postgresql@17` runs the dev cluster on **5433**:
  `DATABASE_URL=postgres://edwin@127.0.0.1:5433/pollen`.
- `PORT` is set in the shell environment and the server rejects it alongside
  `BIND_ADDRESS`, so the API is started with `env -u PORT`.

## Cutting the noise

A pass over everything on screen, keeping only what helps the reader decide.

- **The rail carries a size and nothing else** until a choice goes off the
  standard path. It used to show a verdict banner, four counters, and the full
  consequence ledger — nine cards before a single question was answered. The
  ordinary requirements that follow from a supported setup belong on the
  artifact, not beside the form.
- **The verdict renders only when there is a problem.** A banner announcing that
  all is well is noise on every artifact that has no problem, and it was doubling
  its own message in a subtitle.
- **One signal per state.** An assumed answer showed a chip, a muted tick, and a
  sentence of explanation; it now shows the muted tick and the word "Assumed".
  The "For BES to confirm" chip is gone — the artifact lists the open items.
- **Section blurbs are gone**, and the `blurb` field with them, so nothing
  carries dead data. Headings group the questions on their own.
- **Ruleset copy trimmed.** Help text that restated its own label, notes that
  duplicated the option's warning, and reassurance ("rough numbers are fine")
  are cut. Where a note and a warning said the same thing, the warning stays,
  because it appears at the point of choice.

## Second simplification pass

- **Backups and telemetry are now assumed**, joining the other blessed-path
  defaults. DNS is the only question left that is never guessed: who owns the
  domain varies too much between clients. Answering the two sizing questions
  now yields 12 assumptions and 3 open items.
- **Assumed answers render exactly like chosen ones.** No chip, no muting, no
  "(assumed)" suffix in the record. The default is the answer unless the user
  changes it; the artifact's Assumptions section keeps the record of which were
  filled in.
- **Central server is asked before the facility split**, and the split is
  labelled as being about facility servers so the two do not read as the same
  question.
- **Unsupported platforms are no longer offered.** The "something else" OS
  option and its blocking rule are gone; a configuration BES cannot support is
  not something the tool should invite.
- **Upgrade cadence** drops the vague "as needed" and defaults to every two
  months. The "infrequent upgrades" advisory now fires only for cadences slower
  than the default, so the blessed path raises no callout at all.
- **The technical section is a full-width control** with a chevron, reading
  "Answer more for a more accurate plan", rather than a heading with a small
  Show button beside it.
- **No em-dashes anywhere**, in copy or comments, and the en-dash cost ranges
  read as "hundreds to thousands per year". Each was rewritten in context
  rather than swapped for a single substitute.

## Open: replacing the three-way percentage mix

Three sliders that redistribute against each other are awkward to operate, and
the two commonest answers (all cloud, all client hosted) should not require
touching one at all. Mocked up at
`.workhorse/design/mockups/c1/where-it-runs-options.html`:

- **A.** Three presets, with a single two-ended slider revealed only for "a mix".
- **B.** One row of five bands, no slider and nothing nested. Reuses the band
  control the sizing questions already use.
- **C.** A's presets, with the mix resolving to three discrete buttons.

Any of them still produces a share per class, so the engine's `HasShare`
conditions are unaffected and no rule changes.

Splitting Tamanu Iti into its own question is the larger change: the mix drops
to two classes, and the rules currently keyed on `HasShare("hosting_mix", "iti")`
move to the new question. Iti is arguably not a proportion of the same axis
anyway, since it is a way of running a facility site rather than a place.
