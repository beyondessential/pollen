# View catalogue

How pollen's components compose into screens. The SPA has two real screens — the
**wizard** and the **artifact sheet** — plus transient landing/loading states. Routing
lives in `web/src/App.tsx`; the screens in `web/src/components/` and
`web/src/routes/`. Components referenced here are defined in `components.md`.

## Shell

Every route renders beneath a persistent sticky **topbar** (brand left, "Start a new
plan" right). Below it, a route may render a full-width **update/resume bar** before its
content. The body sits on `--paper`; content surfaces are white.

## Wizard (`/a/:id`, draft status)

The plan-building screen — component `Wizard`, inside a `.frame` two-pane layout.

- **Rail (288px, sticky).** Top-to-bottom: the **verdict banner**, the three **meters**
  (Size · Custom · Blocking), a "Consequences" eyebrow, then the **ledger** — every
  triggered consequence as a **consequence card**, sorted most-severe first (Blocking →
  Non-default → Default), stable within a severity by ruleset order.
- **Main.** The visible **question cards** in order, each rendering as a choice list, a
  band selector, or a multi-select depending on its kind, with optional contextual
  `.guide` notes. It closes with the **action bar**: a hint until every visible question
  is answered, then the primary **Finalise** button.

Answering a question patches the plan and re-evaluates on the server; the choice is
reflected optimistically before the response lands. Below 880px the rail drops away and
the questions take the full width.

## Artifact sheet (`/a/:id`, finalised status)

The finalised, printable plan — component `Artifact`, inside a single centred
**artifact sheet**.

- **Head** — "<Size> deployment" title, a facts row (topology, region), and mono meta
  (config hash, date).
- **Verdict** — the **verdict banner** in its `big` form.
- **Controls** — the audience/topic **toggle group**, a **search input**, and ghost
  actions: **Copy link**, **Download PDF**, **Make changes**.
- **Grouped consequences** — **sheet sections**, one per audience (Client IT · BES
  technical · Record) or per topic depending on the toggle, each holding its matching
  **consequence cards**; empty groups are dropped and a no-match line shows when search
  excludes everything.
- **Decision record** — a closing **record** table of every question and its answer.

"Make changes" forks the plan into a new version (opened in a fresh tab) so the
finalised artifact is never mutated in place. "Download PDF" resets grouping to
by-audience, clears search, and triggers the browser print path (see the print rules in
`design-system.md`).

## Transient states

- **Landing (`/`)** — component `NewApplication`: a **splash** ("Starting a new
  deployment plan…") that creates a draft then replaces the URL with the draft's id. On
  failure it shows the error inline in the splash.
- **Loading / error (`/a/:id`)** — a **splash** while the plan loads or if it fails.
- **Resume offer** — on a fresh, untouched draft with a prior plan on record, a
  **resume bar** offers to pick up where the user left off (Resume / Dismiss). It is
  suppressed when an update/preview bar is already showing, and disappears once a choice
  is made.
- **Not found (`*`)** — a **splash** reading "Not found."

## Related

- `design-system.md` — tokens and principles
- `components.md` — the components these views compose
