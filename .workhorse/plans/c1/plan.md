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

## Hosting: presets instead of a percentage mix

Three sliders redistributing against each other were awkward, and the two
commonest answers should not require touching one at all. Settled on presets
with a discrete follow-up:

- **`hosting_where`** asks all BES cloud, all client hosted, or a mix.
- **`hosting_balance`** appears only for a mix, as three ordered bands (mostly
  BES cloud, about half, mostly client hosted). The engine reads presence and a
  rough proportion, so finer resolution than this buys nothing.
- **`iti_use`** is its own question, appearing once something sits outside BES
  cloud. Iti is a way of running a facility site rather than a place, so
  modelling it as a third slice of the same axis conflated two things.

The `Mix` question kind, the `Answer::Mix` variant, the `HasShare` condition and
the slider control are all deleted rather than left unused. Presence is now
expressed with `Equals` against the preset.

**Sites on an appliance have nothing to provision.** When every non-cloud site
runs an Iti, the operating system and provisioning questions disappear, because
Iti is a fixed ARM64 appliance. The client network requirements still apply.

Copy: BES cloud is described as "hosted by BES in a secure AWS data centre"
rather than by what BES manages, since BES often manages backups and monitoring
for client-hosted deployments too.

## The model schema is part of a binding

Removing the `Mix` question kind broke every plan bound to a ruleset that used
it, including a finalised one, with a 500 out of serde. The lesson is bigger
than the fix: a stored ruleset is frozen content, but the engine reads it
against its own model, so **the model is part of what a binding depends on**.
Withdrawing a variant is the schema equivalent of reusing a stable id, and it
breaks the immutability the artifact lifecycle promises.

Written into the spec as "the engine's model is append-only", because the
lifecycle guarantees depend on it and nothing recorded it.

Two things changed in code:

- A ruleset that will not parse now reports a conflict naming the problem,
  rather than a 500 leaking serde internals out of a public-facing tool. It
  should never fire; it exists because the consequence of it firing is bad.
- A test covers that path with a deliberately unreadable stored ruleset.

The eight already-unreadable dev plans were deleted, since no code path could
load them. Pre-production, deleting `Mix` was still the right call; once the
tool is live, that option closes.

## Spec caught up with the build

The implementation had run ahead of `.workhorse/specs/wizard/onboarding.md`,
which still described a flow where every visible question had to be answered.
Added, in the file's own prose style rather than Workhorse's default checkboxes:

- **The question flow**: sections ordered least technical to most, the sizing
  questions as the only required ones, and the technical section arriving
  collapsed and labelled with what opening it offers.
- **Required, assumed, and open questions**: the three ways a question resolves,
  the test for which a question is authored as, and the guarantee that nested
  assumptions are applied until nothing further resolves.
- **Visibility**: an open question gates nothing, so questions behind it stay
  hidden rather than being asked against an unknown.
- **Artifact lifecycle**: finalising is refused only on required questions, and
  an interim artifact is immutable like any other, so settling its gaps is a new
  version.
- **Outputs**: the finalised view and the PDF both place open questions ahead of
  the detail that rests on them, and list what was assumed.

Em-dashes swept from the spec at the same time.

## Test cases

`.workhorse/test-cases/c1/overview.md`: 30 scenarios covered by automated tests,
11 still owed and all of them interface behaviour that wants a person looking at
it. The unticked boxes are coverage outstanding, not scenarios dismissed.

## The artifact page

- **Finalising now lands at the top.** The artifact replaces the form on the
  same URL, so the reader was left wherever the finalise button had been,
  partway down a document they had not seen the start of. A link naming a
  section is still honoured.
- **One control cluster, not two.** Copy link, PDF and the new-version action
  moved into the header, which had been holding only a date and a config hash.
- **The config hash is gone.** It is not one of the recognition facts the header
  owes a reader (spec WIZ), and a build identifier reads as noise on a document
  that goes to a client.
- **Severity was drawn twice** on every consequence card, as a coloured bar down
  the side and again as a dot beside the title. The bar stays.
- **Search appears only when there is enough to search.** A plan on the standard
  path produces around ten consequences across three sections, which a reader
  takes in at a glance.

DNS is now assumed BES-managed, so the domain chain resolves through to a name
on tamanu.app without the reader opening the technical section. It can still be
declined. Open items on an untouched plan are down to the mobile count and
integrations.

## The artifact reads as a checklist

The by-audience / by-topic toggle is gone. Two views of the same data was a
control the reader had to think about before reading anything, and the topic
view duplicated a grouping the audience view already served. The spec no longer
claims two groupings, and the topic label and ordering maps went with it.

Each consequence is now an item built from the form's own vocabulary: white
surface, hairline border, and a leading marker box, the same shape as a choice
in the questionnaire. It carries more room than the old card, which packed a
title, prose and three tags into ten pixels of padding beside a coloured bar.
The marker carries severity: neutral and empty for an ordinary requirement,
filled and ticked for anything off the standard path.

Audience groups are collapsible, reusing the expander the technical section
already uses, so the two surfaces share one idiom. Groups arrive open, because
collapsing is for skipping past a group addressed to someone else, never for
hiding content from a reader who has not looked yet.

A collapsed group is hidden with CSS rather than left unrendered, so printing
carries the whole record however the reader reached the print dialog. The
browser's own print command is as valid a route as the button.

## The tick box is a real control

The marker on each item was a severity indicator wearing the shape of a
checkbox, which invited exactly the wrong reading. It is now what it looks like:
the reader ticks an item off as the work gets done.

Where the state lives follows from the artifact being immutable and its link
being held by several people at once. Progress is the reader's own, so it is
kept in their browser, never stored against the artifact, and never sent to the
tool. The printed record omits the boxes: the PDF is the record, not a snapshot
of someone's progress.

**Every item gets a box.** Restricting them to client requirements was too
clever by half: "Make DNS resilient to network partitions" is advisory, and is
plainly still something the client does. Ticking means "I have dealt with this",
which applies to doing the work, noting a consequence, and resolving a conflict
alike, so there is no rule to explain and nothing to mis-tag.

The tagging fixes stand on their own merits, because the spec defines a
requirement as what the client's IT team is required to do. Six consequences
described what a choice costs rather than anything anyone does, so they now sit
under the record rather than among the client's actions:

- Client hosts the central server
- A few days of backups only
- Upgrading less often than support covers
- No DNS: all-local, plain HTTP
- No telemetry: support, backups, and mobile are opted out
- DNS: a dedicated BES-owned domain

A seventh moved for the same reason: `dns-bes-subdomain` says outright that
there is nothing for client IT to set up, so it is advice to BES.

The type and status badges are gone, and so is the severity colouring on items,
along with the label maps, the severity palette and the shared tag component
that only they used. Severity now lives solely in the viability callout, which
expands to name the choices behind its count rather than only totalling them.
