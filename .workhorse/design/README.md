# Pollen design library

The design library for pollen — the public-facing Tamanu deployment onboarding wizard.
It is the source of truth for the product's visual language, reverse-engineered from the
shipped SPA (`web/src/`) so that mockups and UI work can be grounded in the real,
BES-branded identity rather than reimagined from scratch.

Pollen's identity is **BES International**: a navy that carries structure (`#313f6a`)
and a sky blue that carries interaction (`#009cea`), on cool off-white paper, set in
Space Grotesk / Inter / IBM Plex Mono. The generic Workhorse design defaults (warm stone
greys, burnt orange) do not apply here.

## Contents

- **[`design-system.md`](design-system.md)** — the single source of truth: colour,
  typography, spacing, radius, elevation, motion, print, and design principles
- **[`components.md`](components.md)** — catalogue of the shipped UI components, each
  with its CSS classes and visual rules
- **[`views.md`](views.md)** — how those components compose into the wizard, the
  artifact sheet, and transient states
- **`mockups/`** — per-card HTML mockups produced during spec/design conversations
  (point-in-time artefacts, not canonical components)

## Using and maintaining it

- When generating a mockup or building UI, source visual language from the shipped
  implementation first, then this library — and where the two disagree, this library
  represents the agreed current direction. Keep it in step with `web/src/app.css` and
  `web/src/theme.ts` as the design evolves.
- The library documents what the system **does** look like, as a coherent snapshot — not
  a changelog of how it got here.
