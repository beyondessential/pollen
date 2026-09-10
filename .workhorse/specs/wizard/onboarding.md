---
id: WIZ
---

# Deployment onboarding wizard

A guided configuration tool that walks a BES team member and a prospective client through the technical decisions required to stand up a new Tamanu deployment.
At each decision it surfaces the consequences of leaving the blessed path (the default, fully-supported configuration) in language a non-technical reader can follow: added cost, degraded operations, lost capabilities, or unsupported configurations.

The tool produces a permanent, shareable artifact that serves three readers at once: the client's IT team (what they must do: ports, outbound endpoints, DNS, region), the BES technical team (the decisions encoded so setup is done correctly), and a record of the choices made, including an acknowledgment of any non-default paths the client opted into.

The artifact is technical, not legal.
It is a record, not a signed instrument.
Contracting may reference it downstream, but the tool does not enforce that, and legal sign-off is kept separate.

## Audience and access

In its first form the BES side drives the tool, working through it with or on behalf of a prospective client.

The tool is public-facing, reachable over the open internet at its own hostname, with no authentication on either drafts or finalised artifacts.
An artifact is addressed by an unguessable identifier in its URL, and that identifier is the only thing protecting it.
Nothing stored is sensitive: no health data, no personal data, and no client or deployment name (see [Data and confidentiality](#data-and-confidentiality)).

## What it is not

The tool surfaces these boundaries to the user, not only records them here.

- **It does not yet scope BES-hosted integrations.**
  If the client wants BES to host an integration (for example a LIMS or pharmacy system, open source and hosted by BES for some clients), the tool records the intent and emits a referral: "this is possible but must be specced in a separate technical conversation."
  Scoping such hosting within the tool is a later addition; for now the hosting detail is resolved in that separate conversation rather than captured here.
- **It does not map individual facilities.**
  The tool operates at planning grain, not provisioning grain.
  It captures the shape and scale of a deployment, not which specific facility is hosted where; that is determined later by the BES technical team.
- **It does not capture free text, names, or notes.**
  Every field is structured or enumerated.

## The question flow

The flow is ordered so that a reader who is not technical can stop early and still leave with something useful.

Questions are grouped into named sections presented in a fixed order, running from the least technical to the most.
The questions that size the deployment come first, and are the only ones that must be answered.
The most technical section arrives collapsed, labelled with what opening it offers rather than with what it contains, so it reads as an invitation to sharpen the estimate rather than as work still owed.
Leaving it collapsed is never a barrier, because every question inside it either carries an answer the engine assumes or can be declined.

## Rule engine model

The tool is driven by a declarative ruleset, not by hard-coded question logic.
The ruleset is data the engine evaluates; this section describes the model the ruleset expresses.
The concrete questions, options, and consequences are not enumerated here: they are defined in the ruleset and documented where it is authored.

Every captured item and every consequence carries up to three independent tags.

### Severity

- **default**: the blessed path; no callout.
- **non-default acknowledgment**: possible but off the blessed path; the client is opting in and accepts the attached consequences.
- **blocking**: internally contradictory or unsupported; will not work as specified, and something must change.

The tool always produces an artifact, even for contradictory input.
A conflict short-circuits the *verdict*, a prominent callout naming the conflicts and stating that the configuration is very likely not possible as specified, rather than the *recording*: the full picture is still captured below the verdict.
The engine evaluates conditions across fields, both as forward guidance during the flow and as a final consistency check when the artifact is finalised.

### Pricing and support commitments

What a configuration costs, and what BES can commit to supporting, are decisions for the pricing and partnerships team rather than facts the artifact settles.

A choice that moves either raises its own item addressed to that team, so they read one list rather than the whole record.
Such an item is work on the standard path, not an acknowledgement: the client accepting a degraded arrangement and BES pricing it are two different things about one choice, so each is its own consequence.
Every plan has something to price, because hosting the derived size band costs something even on the blessed path.

### Consequence type

A single choice may carry several.

- **cost**: that the choice moves what the deployment costs.
  A consequence may carry a magnitude tier and an indicative band where a real one is known, but never a quote, and never a figure nobody has costed.
  What a configuration actually costs is worked out by the pricing and partnerships team, from the items addressed to them.
- **operational impact**: for example slower incident response, manual failover, reduced retention, extended upgrade downtime.
- **capability loss**: for example no Tupaia connection, no clone-upgrade testing.
- **support status**: supported, supported with special arrangement, or unsupported.

### Status

- **requirement**: contractual; what the client's IT team is required to do.
- **advisory**: planning information or recommendation; non-binding.
- **referral**: handled in a separate conversation; the artifact flags the follow-up.

### Triggering

A requirement or consequence appears when its trigger condition holds.

- **Presence of a class.**
  An answer, or the presence of a class within a multi-select mix, sets a flag that makes a block appear.
  A requirement that depends on a kind of facility being present (for example any on-premises facility at all) is triggered by the presence of that class in the mix, not by any individual facility.
- **Cross-field conditions.**
  Conditions spanning several answers are evaluated both for blocking conflicts and for forward guidance, so an early intent can constrain a later question.

The artifact renders the union of every triggered block, grouped for the reader.
Authored prose (consequence detail, question help, option notes, guidance) may
carry limited inline markup: links, which open in a new tab, and light emphasis,
so it can point to further documentation.

The ruleset also carries a compute requirement profile for each class of server or device, each gated by the same trigger conditions.
The engine surfaces the profiles whose class is present in the deployment, which the artifact renders as the [Compute requirements](#compute-requirements).

### Visibility and forward guidance

The engine shows a question only when its precondition holds, and hides it otherwise; a precondition is a presence-of-class flag or a cross-field condition.
An open question asserts nothing, so a question it would gate stays hidden rather than being asked against an unknown.
When an earlier answer will constrain a later question, the engine warns forward at the point the constraint is set, before the later question is reached.
At finalise, the engine re-checks every cross-field condition as a final consistency pass, so a conflict reached by pushing past a warning is still caught.

### Required, assumed, and open questions

A question the reader cannot answer must not block the estimate.
Every question therefore resolves in one of three ways, decided where the ruleset is authored.

- **Required.**
  A question offering neither an assumed answer nor a way to decline must be answered.
  Only questions with no blessed-path answer are authored this way, because there is nothing the engine could assume on the reader's behalf.
- **Assumed.**
  A question left blank takes its authored blessed-path answer, and the artifact records that it was assumed rather than chosen.
  An assumed answer is otherwise indistinguishable from a chosen one: it triggers the same consequences and constrains later questions the same way.
- **Open.**
  A question the reader declines, or leaves blank where declining is offered, asserts nothing.
  No consequence fires either way from it, and it is carried on the artifact as an item still to be settled.

A question is authored as assumable when the blessed path is overwhelmingly what happens and a wrong guess would barely move the estimate.
It is authored as declinable when the answer varies enough between clients that assuming one would misstate the estimate rather than approximate it.

Assuming an answer can reveal a question that carries an assumed answer of its own, so the engine keeps applying them until no further question resolves.
Only questions still visible once that settles are reported as assumed.

## Artifact lifecycle

An artifact is either a **draft** or a **finalised** version, and a finalised version is immutable.

- **Draft.**
  A draft is resumable at a URL carrying an unguessable identifier, and is editable.
  It is persisted, so the same URL reopens the same in-progress artifact.
  The browser also keeps a short, bounded list of recently-touched artifacts (their identifiers and recognition facts only, never identifying data); starting a fresh plan offers to resume the most recent, until the fresh plan itself carries a decision.
- **Finalise.**
  Finalising is refused only while a required question is unanswered; open questions do not stand in its way.
  Finalising produces a permanent, immutable artifact at its own identifier URL.
  This is the canonical artifact.
  A finalised artifact is frozen against the exact ruleset it was finalised under (see [Content-addressed binding](#content-addressed-binding)) and always renders against that frozen ruleset, so a later change to the rules never alters an already-finalised verdict.
- **Interim.**
  A finalised artifact carrying open questions is interim: complete and shareable, but marked as resting partly on gaps rather than wholly on answers.
  It is immutable in the same way as any other finalised artifact, so settling its open questions produces a new version rather than editing it.
  An artifact with no open questions carries no such marking.
- **Versioning.**
  A finalised artifact cannot be edited in place.
  Making changes spawns a new finalised version at a new URL, carrying lineage back to its predecessor.
  This prevents an artifact shifting under someone who already holds the link, and both BES and the client may be holding it.

## The ruleset

The ruleset is data the engine loads, not logic compiled into the tool.
The ruleset is a single document.

### Content-addressed binding

The identity of a ruleset is the hash of its normalized content.
A ruleset is stored once and referenced by that hash; two identical rulesets share one stored copy, and any change yields a new hash.

An artifact records the user's answers together with the hash of the ruleset it is bound to.
Finalising freezes that binding forever: the artifact always evaluates against the exact ruleset content identified by the bound hash, never against whatever the current rules happen to be.

### Production ruleset over the bundled default

The engine ships with a bundled copy of the ruleset, but the live default is whatever the configured repository's production branch (by default `main`) carries: the daemon checks that branch on startup and periodically, and when it holds a different, valid ruleset, adopts it as the default new drafts bind.
So a ruleset change reaches production by merging to that branch, with no engine rebuild or redeploy.
The bundled copy is the fallback: if the branch is unreachable or its ruleset invalid, the current default stays in place.
Finalised artifacts are unaffected, each staying frozen against its bound hash, and a newer default simply surfaces the "new version available" affordance on older plans.

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
Binding the default ruleset, the common path with no branch named, makes no request to the source repository on the request path: it binds the in-memory default. That default is refreshed from the production branch (above) by a background poll on a fixed schedule, so source requests are bounded by the poll interval, not by user traffic.

### Resolution and lifecycle

- **First load.**
  Opening the tool, with or without a named ruleset branch, creates a new draft.
  Any named branch is resolved and its hash bound into the new draft, and the URL then collapses to the draft's own identifier, because the ruleset is recorded against the draft, not carried in the URL.
- **Update is a fork, never a mutation.**
  Naming a ruleset branch again on an existing artifact's URL surfaces a "new version available" affordance.
  Accepting it spawns a new draft bound to the new ruleset hash, with lineage back to the predecessor, and leaves the predecessor, draft or finalised, untouched.
  This works mid-draft and on a long-finalised artifact alike.
- **A migrated artifact lands as a new draft, never auto-finalised.**
  If moving to a new ruleset dropped an answer or surfaced a newly-required question, auto-finalising would freeze an unreviewed guess.
  Landing as a draft flags the gaps for a human; a clean migration is then one extra step to re-finalise.

### The engine's model is append-only

A stored ruleset is frozen content, but the engine reads that content against its own model, so the model is part of what a binding depends on.
Withdrawing a question kind, condition, or tag value the engine once accepted makes every artifact bound to a ruleset using it unreadable, including finalised ones that are guaranteed immutable.
The stable-id discipline applied to questions and options therefore extends to the model itself.

Values the model accepts are added, never withdrawn or repurposed, so any ruleset the tool has ever stored still loads.
An artifact whose bound ruleset cannot be read reports that this version of the tool cannot render the plan, rather than failing as an internal error.

### Stable-id migration

Every question and every option in the ruleset carries a permanent identifier that is never reused or repurposed, and answers are stored by that identifier, never by position or label.

Moving an artifact's answers to a new ruleset is then a set comparison over identifiers:

- an identifier present in both rulesets carries its answer over (a changed label or changed consequence is a re-evaluation against the new rules, not a migration problem);
- an identifier removed in the new ruleset drops its answer, flagged as no longer applicable;
- an identifier new in the new ruleset appears unanswered, flagged as a new question.

This comparison is the "what changed" summary the user sees on update: newly-required questions, dropped answers, and any verdict that changed because the logic changed.

## Outputs

### Finalised web view

The canonical artifact is a live page.
Warnings lead it: what the client is opting into by leaving the standard path is met before anything else.
The compute requirements follow (see [Compute requirements](#compute-requirements)), being the answer the reader came for.
Beneath those, under a **Next steps** heading that frames them as who does what, it presents the remaining consequences in full, in three groups: what the client's IT team has to do, then what the BES technical team sets up, then what the BES pricing and partnerships team has to price or commit to.
An item states the action it asks for, so a group of required actions reads as a list of work rather than a list of observations.
A choice off the standard path that also asks something of a team produces two items: the work, which sits with that team's actions, and the acknowledgement of what the choice costs, which sits with what is being opted into.
Nothing off the standard path appears in a group of actions, because an acknowledgement is not something anyone does.
Warnings arrive collapsed, carrying the same off-default colour they carry everywhere else: they are context rather than the work itself, and a conflict that stops the configuration working is raised by the viability callout regardless.
Each group can be collapsed, so a reader can skip past the groups addressed to someone else.
A section can be linked to directly.

An item in a group of work carries a box the reader ticks off as they do it.
A warning carries none: there is nothing to do about it beyond having read it, and a box would invite it to be treated as a task.
Ticking is the reader's own progress, held in their browser and never against the artifact: a finalised artifact is immutable, and its link is held by several people who are not working through the same list.
The printed record omits the boxes for the same reason.

An interim artifact is marked as interim, and lists its open questions before the detail, so a reader meets what is still unsettled before reading what was decided.
Every assumed answer is listed too, so a reader can see what the tool filled in on their behalf and correct any of it in a new version.

Its header surfaces non-identifying recognition facts so one artifact is distinguishable from another without any free text or name: the size band, the topology shape, the version number, and the creation date, enough that "the medium hybrid, version two" reads differently from "the tiny all-cloud". The size is the derived band, never the raw figures behind it.
A fact the engine assumed reads the same as one the reader chose, because it is the answer until they change it.
Settings that exist to be acted on, such as the hosting region, appear with the work they imply rather than in the header.


### Compute requirements

The compute requirements answer a single question for the reader: what would they have to provide themselves to run this deployment.
So the artifact states the concrete requirements for each class of server and device the client provisions, and only those.
Each class is presented as its own block: the class name, a short line on who provisions it, a set of labelled spec rows leading with processor, memory and storage and then network and operating system or software, and an optional note.

Which classes appear is driven by the answers, so a reader sees only what their deployment needs someone to buy or provide.
A class BES provisions itself, such as a central or facility server hosted in BES cloud, carries no block, because the client provides nothing for it.
So the central server appears only when the client hosts it; a facility server appears when a client-hosted facility runs its own server rather than a mini-server; the Tamanu Iti mini-server, which the client buys from BES, appears when any site uses one; the user devices staff work at always appear; and mobile devices appear when the deployment has mobile users.

A server's processor, memory and storage scale with the deployment's derived size band, drawn from the recommended per-band figures; its network row is the same at every size.
A row may instead be tied to an answer, so the operating system row names the platform the reader chose rather than listing what is available.

A block can carry hints beneath its rows, and a hint may be tied to an answer the same way.
Guidance about how to provision the figures belongs here rather than among the actions, because it qualifies the figures themselves: that a virtual machine may sit slightly under them where its host has room to grow, or that physical hardware should be bought above them because it is hard to change later.
The figures are what to aim for either way, and a hint says which direction it departs in, since "smaller" alone would leave a reader unsure whether the figures were the target or the starting point.
Each appears only for the way of provisioning it describes, and a deployment with both kinds gets both.
Such a hint states the guidance without restating the figures, which would contradict the rows above it.
Working out what is needed is the tool's job, so the requirements state it plainly rather than hedging that a larger deployment might need more.
The make of a server is a suggestion, never a requirement: the block leads with the specification a server must meet, not a product to buy.
For the smallest deployments the block advises hosting with BES or using a mini-server rather than buying a server at all, since dedicated hardware rarely pays off at that scale.
Devices that do not scale, such as workstations and phones, state one recommended specification.
Indicative pricing is out of scope here and is settled by the pricing and partnerships team, who already receive an item to price the hosting.

### PDF export

The artifact can be exported to PDF, sectioned by audience, as a static snapshot derived from the same data.
The PDF is something a reader can save and attach to correspondence.
Its sections, in order:

1. **Viability verdict**: any blocking conflicts, at the top.
2. **Open questions**: on an interim artifact, what is still to be settled, before any of the detail that rests on it.
3. **Warnings**: what the client is opting into by leaving the standard path.
4. **Compute requirements**: the specs for each server and device class the deployment uses.
5. **Client IT team, required actions**: ports, outbound endpoints, DNS, remote access, time synchronisation.
6. **BES technical team, setup decisions**: staging, topology, platform, backup and retention, region.
7. **BES pricing and partnerships**: what has to be priced, and what BES can commit to supporting.
8. **Referrals**: items escalated to a separate conversation.
9. **Assumptions**: the answers the engine filled in where the reader left a question blank.
10. **Full decision record**: everything captured.

## Data and confidentiality

The tool stores only structured answers and the ruleset hash they are bound against.
It stores no free text, no client or deployment name, no health data, and no personal data.
Identifying information, meaning whose deployment an artifact describes, is supplied out of band, in the correspondence that carries the link or PDF, and never enters the stored artifact.

A reader's progress through the required actions is held in their own browser and never sent to the tool, so it is not part of the artifact and not shared by its link.

Because no stored field can carry sensitive or identifying data, a leaked artifact URL exposes only a non-attributable technical configuration.
Names, notes, client self-editing, and authentication move together: none can be added without reconsidering this property.
