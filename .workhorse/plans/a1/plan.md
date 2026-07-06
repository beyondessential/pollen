# Set up design library for this repo

Establish `.workhorse/design/` as the design library for pollen, reverse-engineered
from the shipped SPA (`web/src/`). The library is the source of truth future mockup
and UI work sources from, so it must faithfully capture what is *actually* implemented
— the BES-branded blue palette, the Space Grotesk / Inter / IBM Plex Mono type stack,
and the concrete component patterns in `app.css`, `visuals.tsx`, and the routes.

## Source of truth

- Tokens: `web/src/app.css` (`:root`), `web/src/theme.ts`
- Component styling: `web/src/app.css`, `web/src/components/visuals.tsx`
- Composition/views: `web/src/App.tsx`, `web/src/components/Wizard.tsx`,
  `web/src/components/Artifact.tsx`, `web/src/routes/*`
- Fonts: `web/src/main.tsx` (self-hosted @fontsource)

Note: the generic Workhorse design-system defaults (warm stone greys, burnt orange)
do **not** apply here — pollen ships a BES blue identity. The library documents the
real thing.

## Deliverables

- [x] `.workhorse/design/design-system.md` — the SSoT: palette, semantic colours,
      typography, spacing, radii, elevation, motion, print, and voice/principles
- [x] `.workhorse/design/components.md` — catalogue of shipped components (topbar,
      question card, choices/bands, verdict banner, meters, consequence card + tags,
      buttons, update/resume bars, artifact sheet, decision record, splash)
- [x] `.workhorse/design/views.md` — how the components compose into the wizard,
      the artifact sheet, and the transient landing/loading/not-found states
- [x] `.workhorse/design/README.md` — index/orientation for the library

## Notes

- No `.workhorse/specs/` changes: this is a design/tooling card, not a product
  behaviour change. The library is documentation, not a spec.
- Keep tokens single-sourced in prose (name → value → role) so they don't drift
  from `app.css`; cross-reference the file rather than duplicating full CSS.
