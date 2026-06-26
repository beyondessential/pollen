---
id: WIZ
---

# Deployment onboarding wizard

A guided configuration tool that walks a BES team member and a prospective client through the technical decisions required to stand up a new Tamanu deployment.
At each decision it surfaces the consequences of leaving the blessed path — the default, fully-supported configuration — in language a non-technical reader can follow: added cost, degraded operations, lost capabilities, or unsupported configurations.

The tool produces a permanent, shareable artifact that serves three readers at once: the client's IT team (what they must do — ports, outbound endpoints, DNS, region), the BES technical team (the decisions encoded so setup is done correctly), and a record of the choices made, including an acknowledgment of any non-default paths the client opted into.

The artifact is technical, not legal.
It is a record, not a signed instrument.
Contracting and sales may reference it downstream, but the tool does not enforce that, and legal sign-off is kept separate.

## Audience and access

In its first form the BES side drives the tool, working through it with or on behalf of a prospective client.

The tool is a public-facing surface, reachable over the open internet at its own hostname, with no authentication on either drafts or finalized artifacts.
An artifact is addressed by an unguessable identifier in its URL, and that identifier is the only thing protecting it.
This is acceptable because the tool never stores anything sensitive (see [Data and confidentiality](#data-and-confidentiality)): no health data, no personal data, and no client or deployment name.

The tool is wholly separate from the Tamanu fleet's operator and device surfaces.
It shares no data store with them and holds no access to fleet data, so its exposure to the internet does not widen the fleet's attack surface.

## What it is not

These boundaries are surfaced to the user inside the tool, not just recorded here.

- **It does not scope BES-hosted integrations.**
  If the client wants BES to host an integration (for example a LIMS or pharmacy system, open source and hosted by BES for some clients), the tool records the intent and emits a referral — "this is possible but must be specced in a separate technical conversation" — and does not attempt to capture or resolve the hosting detail.
- **It does not map individual facilities.**
  The tool operates at planning grain, not provisioning grain.
  It captures the shape and scale of a deployment, not which specific facility is hosted where; that is determined later by the BES technical team.
- **It does not capture free text, names, or notes.**
  Every field is structured or enumerated.
  This is what keeps the stored data non-sensitive and justifies unauthenticated, id-addressed access.

## Rule engine model

The tool is driven by a declarative ruleset, not by hard-coded question logic.
The ruleset is data the engine evaluates (see [The ruleset](#the-ruleset)); this section describes the model the ruleset expresses.

Every captured item and every consequence carries up to three independent tags.
Three orthogonal tags keep the model compact and able to absorb new requirements without structural change.

### Severity — the viability verdict

- **default** — the blessed path; no callout.
- **non-default acknowledgment** — possible but off the blessed path; the client is opting in and accepts the attached consequences.
- **blocking** — internally contradictory or unsupported; will not work as specified, and something must change.

The tool always produces an artifact, even for contradictory input.
A conflict short-circuits the *verdict* — a prominent callout that the configuration is very likely not possible as specified — not the *recording*: the full picture is still captured below the verdict.
Blocking is rarely a property of one choice in isolation; it emerges from combinations of choices.
The engine therefore evaluates conditions across fields, both as forward guidance during the flow and as a final consistency check when the artifact is finalized.

### Consequence type — the "this is worse" axis

A single choice may carry several.

- **cost** — a relative magnitude tier, optionally with an indicative ballpark band (for example "on the order of hundreds per year" versus "thousands per year").
  Never a real quote; framed as indicative, with sales confirming actual figures.
- **operational impact** — for example slower incident response, manual failover, reduced retention, extended upgrade downtime.
- **capability loss** — for example no analytics integration, no clone-upgrade testing.
- **support status** — supported, supported with special arrangement, or unsupported.

### Status — the technical-versus-contractual line

- **requirement** — contractual; what the client's IT team is required to do.
- **advisory** — planning information or recommendation; explicitly non-binding.
- **referral** — real and acknowledged, but deliberately handled in a separate conversation; the artifact flags the follow-up rather than ignoring it or over-promising on it.

### Triggering

A requirement or consequence appears when its trigger condition holds.

- **Presence of a class.**
  An answer, or the presence of a class within a multi-select mix, sets a flag that makes a block appear.
  Requirements that depend on a kind of facility being present (for example any on-premises facility at all) are triggered by the presence of that class in the mix, not by any individual facility.
- **Cross-field conditions.**
  Conditions spanning several answers are evaluated both for blocking conflicts and for forward guidance, so an early intent can constrain a later question.

The artifact renders the union of every triggered block, grouped for the reader.

## Question flow

The flow follows the natural cascade of the decisions, and earlier intents constrain later questions.
The exact option lists, band thresholds, size mappings, cost tiers, and consequence wording are ruleset data confirmed with BES, not fixed by this spec; what this spec fixes is the set of decision points the v1 ruleset covers and the cross-field relationships the engine must support.

1. **Analytics intent.**
   Whether the deployment connects to the BES analytics platform.
   Asked first because it constrains backups later: connecting means BES processes the data, which requires at minimum a low-retention backup.
2. **Integrations.**
   Which integration categories are wanted, if any (laboratory, pharmacy/warehousing, and room for more), each optionally a named system.
   The presence of any integration raises a planning advisory that the central server is sized up; the count and mix do not scale this.
   A request for BES to host an integration raises a referral.
3. **Sizing.**
   Catchment population, facility count, and mobile client count, each chosen as a band.
   The highest band reached derives the deployment size, which drives server staging.
4. **Topology.**
   Where the central server runs (BES cloud, customer cloud treated as on-premises, or true on-premises), and the mix of facility placement classes present (BES cloud, on-premises bare-metal, on-premises virtualized, or a BES-built appliance).
   The presence of each class unlocks the relevant sections below.
   BES cloud means BES-managed hosting where the client gets application access by default and system access only on request.
5. **Region** — only when any part of the deployment is in BES cloud.
   A default region, an alternate region, or another region; another region carries a cost consequence.
6. **Platform and operating system** — shown per class present.
   BES cloud and the appliance are fixed and informational.
   On-premises offers a preferred architecture, a fully-supported alternative, a phased-out option that carries cost, operational, and support consequences, and an unsupported option that is blocking — the product is open source and the client can run it, but BES cannot access or support it, which removes support, backups, analytics, and clone-upgrade testing.
   On a supported platform, the client is required to use the BES-provided server image set up the BES way.
7. **On-premises server detail** — only when any on-premises facility is present.
   Whether servers are bare-metal (resources hard to change, so provision generously up front) or virtualized (resources can scale, so start small and grow, with a recommendation to take VM-level backups).
8. **Backups** — two distinct dimensions.
   First, capability: whether BES is allowed to take backups at all.
   Second, retention (only when capability is allowed): full, low-retention (process-only), or none (self-managed).
   These are separate questions because clone-upgrade testing and analytics need only the *ability* to take a backup, even with near-zero retention.
   Disallowing backups entirely is blocking when analytics or BES-managed recovery is also wanted, and otherwise a non-default acknowledgment that removes recovery and clone-upgrade testing.
   Self-managed retention is a non-default acknowledgment on its own; the client must re-assure that they run their own backups.
9. **Upgrade cadence** — advisory only, never a contractual term.
   How often the client intends to upgrade.
   A longer cadence raises an advisory: BES releases frequently, so larger jumps mean heavier migrations, more potential downtime, and stronger reliance on clone-upgrade testing — which in turn needs backup capability.
10. **Networking** — a mix of always-present requirements and triggered ones.
    DNS authority (BES controls DNS, or the client does and must then support automatic certificate issuance and a domain-level wildcard); remote access for managed servers (the BES remote-access network is the requirement, anything else is an exceptional, costed special arrangement); time synchronization (an internal time server or outbound access to public time pools); a partition-resilience recommendation when on-premises facilities are present; and a request for facility egress ranges when the central server is in BES cloud and on-premises facilities are present.

### Forward guidance and visibility

The flow hides questions that do not apply and warns forward when an earlier answer constrains a later one.

- The region question appears only when any BES cloud is present.
- The on-premises detail, partition-resilience, and egress-range items appear only when on-premises facilities are present.
- The retention question appears only when backup capability is allowed.
- An analytics intent constrains the backup questions: the tool warns, before the user reaches backups, that backups cannot be fully disabled, and the final consistency check still catches an incompatible combination if the user pushes through.

## Artifact lifecycle

An artifact is either a **draft** or a **finalized** version, and a finalized version is immutable.

- **Draft.**
  A draft is resumable at a URL carrying an unguessable identifier, and is editable.
  It is persisted, so the same URL reopens the same in-progress artifact.
- **Finalize.**
  Finalizing produces a permanent, immutable artifact at its own identifier URL.
  This is the canonical artifact.
  A finalized artifact is frozen against the exact ruleset it was finalized under (see [Content-addressed binding](#content-addressed-binding)) and always renders against that frozen ruleset, so a later change to the rules never alters an already-finalized verdict.
- **Versioning.**
  A finalized artifact cannot be edited in place.
  Making changes spawns a new finalized version at a new URL, carrying lineage back to its predecessor.
  This prevents an artifact silently shifting under someone who already holds the link — both BES and the client may be holding it.

## The ruleset

The ruleset is data the engine loads, not logic compiled into the tool.
This separates two change cadences: changes to the engine are rare and ship with a deploy, while changes to the rules are frequent and ship without one.
The codebase is open, so there is no value in hiding the rules; the ruleset is a single document.

### Content-addressed binding

The identity of a ruleset is the hash of its normalized content.
A ruleset is stored once and referenced by that hash; two identical rulesets share one stored copy, and any change yields a new hash.

An artifact records the user's answers together with the hash of the ruleset it is bound to.
Finalizing freezes that binding forever: the artifact always evaluates against the exact ruleset content identified by the bound hash, never against "the latest rules".

### Preview against repository refs

A change to the rules can be previewed against the live tool before it is merged, by naming a branch of the ruleset's own source repository in the artifact's URL.
Naming a branch resolves it, through that repository's own reference list, to the commit in that repository; the ruleset content at that commit is fetched, normalized, hashed, and bound.
The repository whose references are consulted is fixed by configuration and is the only source a ruleset is ever fetched from.

This resolution is a security boundary and must not be weakened.
The tool is public and its links are forwarded to clients, so an override that fetched a ruleset from an arbitrary location would be a content-injection vector — a client could be handed a link that loads an attacker's ruleset.
Resolving a *branch name* against the configured repository's own references closes this: a branch is intrinsically scoped to that repository's namespace, so a fork's branch cannot be named through the upstream repository's references.
A check that merely inspects whether a supplied URL "looks like" it points at the repository is insufficient, because content URLs can be crafted to appear in-repository while resolving to a fork's commit.

A branch is mutable, which is exactly why naming a branch is preview-only and finalizing binds the resolved content hash rather than the branch name.
The binding chain strips mutability at each step: a branch (mutable, for preview convenience) resolves to content (verified to come from the configured repository), which hashes to an immutable identity, which is stored once and referenced by artifacts.

### Resolution and lifecycle

- **First load.**
  Opening the tool, with or without a named ruleset branch, creates a new draft.
  Any named branch is resolved and its hash bound into the new draft, and the URL then collapses to the draft's own identifier — the ruleset is recorded against the draft, not carried in the URL.
- **Update is a fork, never a mutation.**
  Naming a ruleset branch again on an existing artifact's URL surfaces a "new version available" affordance.
  Accepting it spawns a new draft bound to the new ruleset hash, with lineage back to the predecessor, and leaves the predecessor — draft or finalized — untouched.
  This works mid-draft and on a long-finalized artifact alike.
- **A migrated artifact lands as a new draft, never auto-finalized.**
  If moving to a new ruleset dropped an answer or surfaced a newly-required question, auto-finalizing would silently finalize a guess — the very thing freezing exists to prevent.
  Landing as a draft flags the gaps for a human; a clean migration is then one extra step to re-finalize.

### Stable-id migration

Every question and every option in the ruleset carries a permanent identifier that is never reused or repurposed, and answers are stored by that identifier, never by position or label.

Moving an artifact's answers to a new ruleset is then a set comparison over identifiers:

- an identifier present in both rulesets carries its answer over (a changed label or changed consequence is a re-evaluation against the new rules, not a migration problem);
- an identifier removed in the new ruleset drops its answer, flagged as no longer applicable;
- an identifier new in the new ruleset appears unanswered, flagged as a new question.

This comparison is itself the "what changed" summary the user sees on update: newly-required questions, dropped answers, and any verdict that changed because the logic changed.

## Outputs

### Finalized web view

The canonical artifact is a live page that renders the same underlying data in more than one way.
It can be presented grouped by audience or grouped by topic, and supports searching, expanding and collapsing, and deep-linking to a section.

Its header surfaces non-identifying recognition facts so one artifact is distinguishable from another without any free text or name: the size band, the topology shape, the region, the version number, and the creation date — enough that "the fifteen-facility hybrid in the alternate region, version two" reads differently from "the five-facility all-cloud default-region".

### PDF export

The artifact can be exported to PDF, sectioned by audience, as a static snapshot derived from the same data.
The PDF is something a reader can save and attach to correspondence.
Its sections, in order:

1. **Viability verdict** — any blocking conflicts, at the top.
2. **Client IT team — required actions** — ports, outbound endpoints, DNS, remote access, time synchronization, region.
3. **BES technical team — setup decisions** — sizing and staging, topology, platform, server specifications, backup and retention, the integrations capacity note.
4. **Non-default acknowledgments** — what the client is opting into, with consequences grouped by type.
5. **Advisory and planning** — non-binding recommendations and captured planning data.
6. **Referrals** — items escalated to a separate conversation.
7. **Full decision record** — everything captured.

## Data and confidentiality

The tool stores only structured answers and the ruleset hash they are bound against.
It stores no free text, no client or deployment name, no health data, and no personal data.
Identifying information — whose deployment an artifact describes — is supplied out of band, in the correspondence that carries the link or PDF, and never enters the stored artifact.

This is the linchpin that makes unauthenticated, id-addressed access acceptable: because no stored field can carry sensitive or identifying data, an artifact's URL leaking exposes only a non-attributable technical configuration.
Names, notes, client self-editing, and authentication are a single coherent bundle that move together if they are ever added; none can be added piecemeal without reconsidering this property.
