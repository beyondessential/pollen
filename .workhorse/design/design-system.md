# Pollen design system

The single source of truth for pollen's visual language. Pollen is the public-facing
Tamanu deployment onboarding wizard — a self-contained axum + React/MUI SPA. Its
identity is **BES International**: a navy that carries structure and a sky blue that
carries interaction, set on a cool off-white paper.

These tokens are reverse-engineered from the shipped SPA and mirror
`web/src/app.css` (`:root`) and `web/src/theme.ts`. When the two disagree, this
document represents the agreed direction; keep `app.css` and this file in step.

> The generic Workhorse defaults (warm stone greys, burnt orange) do **not** apply to
> pollen. Everything below is the real, shipped BES-blue identity.

## Principles

- **Structure in navy, interaction in sky.** Navy (`#313f6a`) is for primary actions,
  selected bands, and load-bearing chrome. Sky (`#009cea`) is for selection ticks,
  section eyebrows, links, and informational washes. Never swap their roles.
- **Calm, document-like surfaces.** White cards float on cool paper with a hairline
  border and a soft, low, downward shadow — not heavy drop shadows. The artifact reads
  like a printable record, not a dashboard.
- **Consequence colour is semantic, never decorative.** Green/amber/red map to
  Default / Non-default / Blocking and to nothing else. Don't tint UI with them for
  emphasis.
- **Quiet motion.** A single 4px rise on entrance and 0.14s hover transitions. All
  motion is gated behind `prefers-reduced-motion`.
- **No purple, no gradients, no sparkle.** Flat fills, hairline borders, one accent
  family.

## Colour

### Brand

| Token | Value | Role |
| --- | --- | --- |
| `--navy` | `#313f6a` | Primary brand; primary buttons, selected band, `qtitle` accents, MUI `primary` |
| `--navy-deep` | `#27325a` | Navy hover/pressed; `derive`/`updatebar` prose text |
| `--sky` | `#009cea` | Secondary brand; selection tick, selected choice border, MUI `secondary` |
| `--sky-deep` | `#0784c4` | Sky as text: links, section eyebrows, "start new plan" control |
| `--sky-tint` | `#e7f5fd` | Sky informational wash: selected choice, guide/derive/update bars |

### Ink (text)

| Token | Value | Role |
| --- | --- | --- |
| `--ink` | `#14222b` | Primary text |
| `--ink-soft` | `#485a63` | Secondary text, help copy, ghost-button label |
| `--ink-faint` | `#7e8c94` | Tertiary/muted: meta, notes, disabled hints, rail labels |

### Neutrals

| Token | Value | Role |
| --- | --- | --- |
| `--paper` | `#f4f6f8` | Page background (MUI `background.default`) |
| `--surface` | `#ffffff` | Cards, rail, sheet, controls (MUI `background.paper`) |
| `--line` | `#e3e8eb` | Standard borders |
| `--line-soft` | `#eef1f3` | Subtle dividers, inner separators |

Hover borders on interactive surfaces darken to `#c4d0d6` (a one-off, not a token).

### Semantic — verdict & severity

Each pairs a saturated foreground with a pale background wash of the same hue. Used for
the verdict banner, severity dots/bars, and severity-tinted meters.

| Meaning | Foreground | Background |
| --- | --- | --- |
| Default / on the supported path | `--clear` `#2e7d52` | `--clear-bg` `#e4f1ea` |
| Non-default / off the default path | `--offdef` `#b5791f` | `--offdef-bg` `#fbf1dc` |
| Blocking / not possible as specified | `--block` `#b23a3a` | `--block-bg` `#f7e4e1` |

Severity dot/bar uses the foreground colour; the "Default" severity is the exception —
its dot is `--ink-faint` and its wash is `--line-soft`, so an unremarkable consequence
stays visually neutral.

### Consequence-type tags

Pill tags on consequence cards, one pale hue per type (`visuals.tsx`, `TYPE_COLOR`):

| Type | Foreground | Background |
| --- | --- | --- |
| Cost | `#9a6a00` | `#fbf1dc` |
| Operational | `#1f5fa6` | `#e6eff8` |
| Capability loss | `#8a2e2e` | `#f7e4e1` |
| Support | `#6b3a8a` | `#efe6f6` |

A neutral status tag (`#3a4750` on `#eaedee`) sits alongside them.

## Typography

Three self-hosted families, bundled into the binary (`main.tsx`) — no runtime fetch.

| Token | Stack | Role |
| --- | --- | --- |
| `--font-display` | `"Space Grotesk", system-ui, sans-serif` | Headings (h1–h3), card/section titles, verdict text, meter values, sheet title |
| `--font-body` | `"Inter", system-ui, sans-serif` | Body, controls, labels — the default |
| `--font-mono` | `"IBM Plex Mono", ui-monospace, monospace` | Config hashes, dates, recorded answer values, inline `code` |

Weights bundled: Space Grotesk 400/500/600/700, Inter 400/500/600, IBM Plex Mono
400/500.

### Scale

Type is compact and set in pixels in the shipped CSS. Key sizes, by role:

- **Sheet title** — 26px / 600, Space Grotesk, `-0.02em` tracking
- **Question title** (`qtitle`) — 17px / 600, Space Grotesk, `-0.01em`
- **Meter value** — 18px / 600, Space Grotesk
- **Brand name** — 15px / 500, Space Grotesk
- **Body / choice title / button** — 13–14px, Inter
- **Help & note copy** — 12–13px, Inter, line-height ~1.5
- **Eyebrows** — 10.5–11px, uppercase, letter-spacing 0.10–0.12em, weight 600, in
  `--sky-deep` (content eyebrows) or `--ink-faint` (rail/meta eyebrows)
- **Tags** — 10.5px / 500

Display headings carry slight negative tracking; eyebrows carry wide positive tracking.
Body text never goes below ~12px.

## Spacing & layout

- **4px grid.** Padding and gaps are multiples/near-multiples of 4 (e.g. card padding
  `24px 26px`, choice padding `13px 15px`, gaps of 6/8/10/12/16px).
- **Two-pane frame.** The wizard is a `288px` sticky rail + fluid main, capped at
  `1180px` and centred. Below `880px` the rail is hidden and it collapses to one column.
- **Artifact sheet** is a single centred column capped at `920px`.
- **Sticky chrome.** The topbar sticks to the top (`z-index: 20`); the rail sticks
  below it at `top: 57px`.

## Radius

| Token / value | Applied to |
| --- | --- |
| `--radius` `12px` | Question cards, informational bars |
| `--radius-sm` `9px` | Meters |
| `16px` | Artifact sheet (the largest surface) |
| `10–11px` | Choices, bands, buttons, consequence cards, records |
| `8px` | Small buttons, toggles, search, topbar control |
| `6px` | Selection tick, inline code chips |
| `99px` (pill) | Tags, severity dots |

Larger surfaces take larger radii; controls stay tighter.

## Elevation

Two soft, downward-only shadows layered as a tight contact shadow plus a wide, faint
ambient one — never a hard drop shadow:

- **Card** — `0 1px 2px rgba(20,34,43,.03), 0 12px 28px -22px rgba(20,34,43,.16)`
- **Sheet** — `0 1px 2px rgba(20,34,43,.03), 0 16px 36px -24px rgba(20,34,43,.2)`

Everything else relies on the hairline `--line` border rather than shadow.

## Motion

- **Entrance rise.** Cards and consequence cards animate in with `rise`: opacity 0→1
  and `translateY(4px)`→0 over 0.24s ease, gated behind
  `@media (prefers-reduced-motion: no-preference)`.
- **Hover transitions.** Interactive surfaces transition `border-color` / `background`
  over ~0.14s; buttons over ~0.15s.
- Keep new motion in this register — short, small, opt-out-aware.

## Print

The artifact is designed to print as a clean record (`@media print`): topbar, update
bar, and sheet controls are hidden; the sheet drops its max-width, border, radius, and
shadow; sections/cards/records avoid inner page breaks; the entrance animation is
forced to its finished state so nothing prints greyed-out mid-flight; links print as
plain inherited-colour text.

## Iconography

Icons are minimal inline SVG (lucide-style paths, `stroke="currentColor"`,
`stroke-width: 2`, round caps/joins) rendered by the `Icon` helper in `visuals.tsx` —
nothing is fetched at runtime. The only shipped glyph is a check; add new icons in the
same 24×24 stroked style.

## Inline prose markup

Ruleset-authored copy (help text, notes, consequence detail, guidance) is rendered
through `markup.tsx`, which supports Markdown links (opening in a new tab), `**bold**`,
`*italic*`, and `` `code` ``. Links render in `--sky-deep` underlined with a 2px offset;
code renders in `--font-mono` at 0.9em. It renders to React nodes, never raw HTML, and
only accepts `http(s)` link targets.

## Related

- `components.md` — the shipped component catalogue
- `views.md` — how components compose into screens
