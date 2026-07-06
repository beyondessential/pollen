# Component catalogue

The shipped UI components of the pollen SPA, as implemented in `web/src/app.css` and
`web/src/components/visuals.tsx`. Each entry names the component, its CSS class(es), and
the visual rules that define it. Tokens referenced here are defined in
`design-system.md`.

When building new UI, compose from these first and match their register; only author
fresh patterns when nothing here fits.

## Topbar & brand

`.topbar`, `.brand`, `.brand-logo`, `.brand-rule`, `.brand-name`, `.topbar-new`

A sticky white bar with a bottom hairline, `14px 28px` padding. Left: the BES logo
(26px rounded square) · a 1px vertical rule · the product name "New Tamanu" in Space
Grotesk 15px/500. Right: a "Start a new plan" text control — `--sky-deep`, 13px/500,
hairline-bordered, `7px 13px`, that fills with `--sky-tint` on hover. A mono caption
(`--ink-faint`, 11px) may sit in the bar for build/version info.

## Question card

`.card`, `.qtitle`, `.qhelp`, `.section-eyebrow`, `.guide`

The primary content container in the wizard. White surface, `--line` border,
`--radius`, `24px 26px` padding, card shadow, `16px` bottom margin. Enters with the
`rise` animation.

- **`.qtitle`** — Space Grotesk 17px/600, `-0.01em`.
- **`.qhelp`** — 13px `--ink-soft`, line-height 1.5; renders inline markup.
- **`.section-eyebrow`** — uppercase 11px/600 `--sky-deep` label that groups cards into
  sections.
- **`.guide`** — an inline `--sky-tint` note inside a card (contextual guidance): sky
  wash, `#bfe4f8` border, `--navy-deep` text, `--radius-sm`-ish 10px.

## Choice list

`.choices`, `.choice`, `.choice.on`, `.choice-tick`, `.choice-title`, `.choice-note`

Vertical list of selectable options (single- or multi-select). Each `.choice` is a
full-width left-aligned button: white, hairline border, 11px radius, `13px 15px`
padding. Hover darkens the border and washes to `#fbfdfe`. Selected (`.on`) takes a
`--sky` border and `--sky-tint` fill.

- **`.choice-tick`** — 18px rounded-square, hairline border; when selected, fills
  `--sky` and shows a white check (the `Check` inline icon).
- **`.choice-title`** — 14px/500; **`.choice-note`** — 12px `--ink-faint`, supports
  inline markup.

Multi-select honours exclusivity (an exclusive "none of these" option clears the rest,
and vice versa) — see `Wizard.tsx`.

## Band selector

`.bandrow`, `.band`, `.band.on`

A compact segmented control for ordinal/scale answers: a 4-column grid (2 columns below
`560px`) of small buttons — 12.5px `--ink-soft`, hairline border, 10px radius. Selected
(`.on`) inverts to a solid `--navy` fill with white text. Bands select in **navy**
(structural quantity); choices select in **sky** (interaction) — this distinction is
intentional.

## Derived-value note

`.derive`

A summary readout shown after inputs (e.g. a derived deployment size): `--sky-tint`
wash, `#bfe4f8` border, 10px radius, `--navy-deep` text; the emphasised value uses the
display font.

## Verdict banner

`.verdict`, `.verdict-big`, `.verdict-t`, `.verdict-s` — component `VerdictBanner`

The headline status of a plan. A pill-radius (11px) block whose foreground/background
come from the verdict's semantic pair (clear / off-default / blocking). `.verdict-t` is
the display-font title; `.verdict-s` the muted subtitle with the count summary.
`.verdict-big` enlarges it for the artifact head. Before any choice is made it shows a
neutral `--line-soft` "Nothing recorded yet" state.

Verdict → copy:

- **Blocking** — "Not possible as specified" · _N blocking conflicts — something must
  change._
- **Non-default** — "Possible, with acknowledged off-default choices" · _N choices off
  the default path. This will be harder to support._
- **Default** — "On the default, supported path" · _No off-default choices recorded._

## Meters

`.meters`, `.meter`, `.meter-k`, `.meter-v`

A row of three equal stat tiles in the rail (Size · Custom · Blocking). Each is a pale
`--paper` tile with a `--line-soft` border and `--radius-sm`: a tiny uppercase key
(`.meter-k`, 9.5px `--ink-faint`) over a display-font value (`.meter-v`, 18px/600). The
Custom and Blocking values turn `--offdef` / `--block` when non-zero.

## Consequence card

`.cons`, `.cons-bar`, `.cons-body`, `.cons-head`, `.cons-dot`, `.cons-title`,
`.cons-detail`, `.cons-tags`, `.tag`, `.cost-note` — components `ConsequenceCard`, `Tag`

The core record of a single triggered consequence, used in both the rail ledger and the
artifact sheet. A white card with a **3px left severity bar** (`.cons-bar`) coloured by
severity. The body carries:

- **Head** — a 7px severity `.cons-dot` and a display-font `.cons-title` (13px/600).
- **Detail** — `.cons-detail`, 12.5px `--ink-soft`, max 70ch, inline markup.
- **Tags** — one `.tag` (pill, 10.5px/500) per consequence type in its type colour, plus
  a neutral status tag; an optional italic `.cost-note` (`--ink-faint`) shows cost
  tier · ballpark.

Card border colour is set to the severity background so neutral consequences recede and
blocking ones read hot.

## Buttons

`.btn`, `.btn.primary`, `.btn.ghost`, `.btn.sm`

- **Primary** — solid `--navy`, white text; hover `--navy-deep`. Used for the single
  key action (Finalise, Update, Resume).
- **Ghost** — white, hairline border, `--ink-soft` label; hover darkens the border.
  Used for secondary artifact actions (Copy link, Download PDF, Make changes).
- Base: inline-flex, `11px 20px`, 10px radius, 14px/500, ~0.15s transition. **`.sm`**
  tightens to `7px 14px` / 13px / 8px radius (used in banners). Disabled drops to 0.5
  opacity.

## Action bar

`.actions`, `.actions-hint`

A right-aligned footer row under the wizard cards holding the primary action, with an
optional left-aligned `--ink-faint` hint ("Answer every question to finalise.").

## Update / resume bar

`.updatebar`, `.updatebar-sub`, `.updatebar-actions` — components `UpdateBar`,
`ResumeBar`

A full-width centred notice below the topbar: `--sky-tint` wash, `#bfe6f9` bottom
border, `--navy-deep` text, 13.5px. Inline `code` chips (ruleset branch names) get a
white background and sky border. Carries one or two `.sm` buttons — a primary action
plus (for resume) a ghost Dismiss. Used for ruleset-preview / update-available prompts
and the "pick up where you left off?" resume offer.

## Artifact sheet

`.sheet`, `.sheet-head`, `.sheet-eyebrow`, `.sheet-title`, `.sheet-facts`,
`.sheet-meta`, `.sheet-controls`, `.sheet-section`, `.sheet-section-title` — component
`Artifact`

The finalised plan as a printable document. A single centred surface (max 920px, 16px
radius, sheet shadow) with:

- **Head** — a faintly-tinted (`#fbfdfe`) band with the display-font `.sheet-title`
  (26px) "<Size> deployment", a `.sheet-facts` row (topology, region) in `--ink-soft`,
  and right-aligned mono `.sheet-meta` (config hash, date).
- **Controls** — a toggle group (`.tg` — By audience / By topic), a search input
  (`.sheet-search`), and ghost action buttons; hidden in print.
- **Sections** — `.sheet-section` blocks separated by `--line-soft`, each led by an
  uppercase `--sky-deep` `.sheet-section-title`, holding grouped consequence cards.

## Toggle group

`.tg-group`, `.tg`, `.tg.on`

Segmented toggle for view grouping. Each `.tg` is a hairline-bordered white pill-ish
button (8px radius, 12.5px `--ink-soft`); the active one inverts to solid `--navy`
white — the same navy-selection language as bands.

## Search input

`.sheet-search`

A flexible hairline-bordered text input (8px radius, 13px), `type="search"`, filtering
consequences live by title/detail.

## Decision record

`.record`, `.record-row`, `.ledger-empty`

A bordered table of every question and its recorded answer, closing the artifact. Rows
(`.record-row`) are `space-between` with a `--ink-soft` label and a mono value; odd rows
get a `#fbfdfe` zebra tint. `.ledger-empty` is the muted empty-state line (e.g. "No
consequences match your search.").

## Splash / status

`.splash`

A centred full-height placeholder in `--ink-faint` 14px for transient states — starting
a plan, loading, load errors, and not-found.

## Related

- `design-system.md` — tokens these components draw on
- `views.md` — how they compose into screens
