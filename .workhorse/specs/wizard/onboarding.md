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

The tool is public-facing, reachable over the open internet at its own hostname, with no authentication on either drafts or finalised artifacts.
An artifact is addressed by an unguessable identifier in its URL, and that identifier is the only thing protecting it.
Nothing stored is sensitive: no health data, no personal data, and no client or deployment name (see [Data and confidentiality](#data-and-confidentiality)).

## What it is not

The tool surfaces these boundaries to the user, not only records them here.

- **It does not yet scope BES-hosted integrations.**
  If the client wants BES to host an integration (for example a LIMS or pharmacy system, open source and hosted by BES for some clients), the tool records the intent and emits a referral — "this is possible but must be specced in a separate technical conversation."
  Scoping such hosting within the tool is a later addition; for now the hosting detail is resolved in that separate conversation rather than captured here.
- **It does not map individual facilities.**
  The tool operates at planning grain, not provisioning grain.
  It captures the shape and scale of a deployment, not which specific facility is hosted where; that is determined later by the BES technical team.
- **It does not capture free text, names, or notes.**
  Every field is structured or enumerated.

## Rule engine model

The tool is driven by a declarative ruleset, not by hard-coded question logic.
The ruleset is data the engine evaluates; this section describes the model the ruleset expresses.
The concrete questions, options, and consequences are not enumerated here: they are defined in the ruleset and documented where it is authored.

Every captured item and every consequence carries up to three independent tags.

### Severity

- **default** — the blessed path; no callout.
- **non-default acknowledgment** — possible but off the blessed path; the client is opting in and accepts the attached consequences.
- **blocking** — internally contradictory or unsupported; will not work as specified, and something must change.

The tool always produces an artifact, even for contradictory input.
A conflict short-circuits the *verdict* — a prominent callout that the configuration is very likely not possible as specified — not the *recording*: the full picture is still captured below the verdict.
The engine evaluates conditions across fields, both as forward guidance during the flow and as a final consistency check when the artifact is finalised.

### Consequence type

A single choice may carry several.

- **cost** — a relative magnitude tier, optionally with an indicative ballpark band (for example "on the order of hundreds per year" versus "thousands per year").
  Never a real quote; sales confirms actual figures.
- **operational impact** — for example slower incident response, manual failover, reduced retention, extended upgrade downtime.
- **capability loss** — for example no analytics integration, no clone-upgrade testing.
- **support status** — supported, supported with special arrangement, or unsupported.

### Status

- **requirement** — contractual; what the client's IT team is required to do.
- **advisory** — planning information or recommendation; non-binding.
- **referral** — handled in a separate conversation; the artifact flags the follow-up.

### Triggering

A requirement or consequence appears when its trigger condition holds.

- **Presence of a class.**
  An answer, or the presence of a class within a multi-select mix, sets a flag that makes a block appear.
  A requirement that depends on a kind of facility being present (for example any on-premises facility at all) is triggered by the presence of that class in the mix, not by any individual facility.
- **Cross-field conditions.**
  Conditions spanning several answers are evaluated both for blocking conflicts and for forward guidance, so an early intent can constrain a later question.

The artifact renders the union of every triggered block, grouped for the reader.
Authored prose (consequence detail, question help, option notes, guidance) may
carry limited inline markup — links, which open in a new tab, and light emphasis
— so it can point to further documentation.

### Visibility and forward guidance

The engine shows a question only when its precondition holds, and hides it otherwise; a precondition is a presence-of-class flag or a cross-field condition.
When an earlier answer will constrain a later question, the engine warns forward at the point the constraint is set, before the later question is reached.
At finalise, the engine re-checks every cross-field condition as a final consistency pass, so a conflict reached by pushing past a warning is still caught.

## Artifact lifecycle

An artifact is either a **draft** or a **finalised** version, and a finalised version is immutable.

- **Draft.**
  A draft is resumable at a URL carrying an unguessable identifier, and is editable.
  It is persisted, so the same URL reopens the same in-progress artifact.
  The browser also keeps a short, bounded list of recently-touched artifacts (their identifiers and recognition facts only, never identifying data); starting a fresh plan offers to resume the most recent, until the fresh plan itself carries a decision.
- **Finalise.**
  Finalising produces a permanent, immutable artifact at its own identifier URL.
  This is the canonical artifact.
  A finalised artifact is frozen against the exact ruleset it was finalised under (see [Content-addressed binding](#content-addressed-binding)) and always renders against that frozen ruleset, so a later change to the rules never alters an already-finalised verdict.
- **Versioning.**
  A finalised artifact cannot be edited in place.
  Making changes spawns a new finalised version at a new URL, carrying lineage back to its predecessor.
  This prevents an artifact shifting under someone who already holds the link — both BES and the client may be holding it.

## The ruleset

The ruleset is data the engine loads, not logic compiled into the tool.
The ruleset is a single document.

### Content-addressed binding

The identity of a ruleset is the hash of its normalized content.
A ruleset is stored once and referenced by that hash; two identical rulesets share one stored copy, and any change yields a new hash.

An artifact records the user's answers together with the hash of the ruleset it is bound to.
Finalising freezes that binding forever: the artifact always evaluates against the exact ruleset content identified by the bound hash, never against whatever the current rules happen to be.

### Preview against repository refs

A change to the rules can be previewed against the live tool before it is merged, by naming a branch of the ruleset's source repository in the artifact's URL.
Naming a branch resolves it, through that repository's own reference list, to the commit in that repository; the ruleset content at that commit is fetched, normalized, hashed, and bound.
The repository whose references are consulted is fixed by configuration and is the only source a ruleset is ever fetched from.

The tool is public and its links are forwarded to clients, so a ruleset fetched from an attacker-controlled location would let a client be handed a link that loads an attacker's ruleset.
Resolving a *branch name* against the configured repository's own references prevents this: a branch is scoped to that repository's namespace, so a fork's branch cannot be named through the upstream repository's references.
A check that inspects whether a supplied URL "looks like" it points at the repository is insufficient, because content URLs can be crafted to appear in-repository while resolving to a fork's commit.

A branch is mutable; naming a branch is therefore preview-only, and finalising binds the resolved content hash, not the branch name.
The binding chain strips mutability at each step: a branch resolves to content (verified to come from the configured repository), which hashes to an immutable identity, which is stored once and referenced by artifacts.

Resolving a branch against the source repository is rate-limited, and a resolved result is briefly cached, so repeated or abusive preview requests do not exhaust the repository host's request quota.
Binding the already-stored default ruleset — the common path, with no branch named — makes no request to the source repository.

### Resolution and lifecycle

- **First load.**
  Opening the tool, with or without a named ruleset branch, creates a new draft.
  Any named branch is resolved and its hash bound into the new draft, and the URL then collapses to the draft's own identifier — the ruleset is recorded against the draft, not carried in the URL.
- **Update is a fork, never a mutation.**
  Naming a ruleset branch again on an existing artifact's URL surfaces a "new version available" affordance.
  Accepting it spawns a new draft bound to the new ruleset hash, with lineage back to the predecessor, and leaves the predecessor — draft or finalised — untouched.
  This works mid-draft and on a long-finalised artifact alike.
- **A migrated artifact lands as a new draft, never auto-finalised.**
  If moving to a new ruleset dropped an answer or surfaced a newly-required question, auto-finalising would freeze an unreviewed guess.
  Landing as a draft flags the gaps for a human; a clean migration is then one extra step to re-finalise.

### Stable-id migration

Every question and every option in the ruleset carries a permanent identifier that is never reused or repurposed, and answers are stored by that identifier, never by position or label.

Moving an artifact's answers to a new ruleset is then a set comparison over identifiers:

- an identifier present in both rulesets carries its answer over (a changed label or changed consequence is a re-evaluation against the new rules, not a migration problem);
- an identifier removed in the new ruleset drops its answer, flagged as no longer applicable;
- an identifier new in the new ruleset appears unanswered, flagged as a new question.

This comparison is the "what changed" summary the user sees on update: newly-required questions, dropped answers, and any verdict that changed because the logic changed.

## Outputs

### Finalised web view

The canonical artifact is a live page that renders the same underlying data in more than one way.
It can be presented grouped by audience or grouped by topic, with every consequence shown in full, and supports searching and deep-linking to a section.

Its header surfaces non-identifying recognition facts so one artifact is distinguishable from another without any free text or name: the size band, the topology shape, the region, the version number, and the creation date — enough that "the fifteen-facility hybrid in the alternate region, version two" reads differently from "the five-facility all-cloud default-region".

### PDF export

The artifact can be exported to PDF, sectioned by audience, as a static snapshot derived from the same data.
The PDF is something a reader can save and attach to correspondence.
Its sections, in order:

1. **Viability verdict** — any blocking conflicts, at the top.
2. **Client IT team — required actions** — ports, outbound endpoints, DNS, remote access, time synchronisation, region.
3. **BES technical team — setup decisions** — sizing and staging, topology, platform, server specifications, backup and retention, the integrations capacity note.
4. **Non-default acknowledgments** — what the client is opting into, with consequences grouped by type.
5. **Advisory and planning** — non-binding recommendations and captured planning data.
6. **Referrals** — items escalated to a separate conversation.
7. **Full decision record** — everything captured.

## Data and confidentiality

The tool stores only structured answers and the ruleset hash they are bound against.
It stores no free text, no client or deployment name, no health data, and no personal data.
Identifying information — whose deployment an artifact describes — is supplied out of band, in the correspondence that carries the link or PDF, and never enters the stored artifact.

Because no stored field can carry sensitive or identifying data, a leaked artifact URL exposes only a non-attributable technical configuration.
Names, notes, client self-editing, and authentication move together: none can be added without reconsidering this property.
