---
id: COST
---

# Internal hosting cost

An internal edition of the tool that shows what a deployment costs BES to host each month.
It has two surfaces sharing one cost model: a calculator that prices a handful of inputs directly, and a priced edition of the wizard that keeps a running cost beside the questions.
The public tool described in [WIZ](../wizard/onboarding.md) shows no figures, and nothing on it links to the internal edition.

## Access

The internal edition lives under a path segment that is unguessable and set in configuration, never in the code or the ruleset.
That segment is the only thing protecting it, in the same way an artifact's identifier protects the artifact.
The cost endpoints answer only a request carrying the same segment, and answer anything else as though they did not exist, so the public tool cannot reach a figure.
With no segment configured, the internal edition is switched off entirely.

- [ ] The calculator opens at the configured path.
- [ ] A new priced plan starts from the configured path, and an existing plan opens priced under it.
- [ ] Moving between plans in the priced edition (resuming, updating, making a new version) stays within the priced edition.
- [ ] A wrong or missing segment shows the tool's ordinary "not found" page.
- [ ] The public wizard, its artifact, and its printed record carry no figure and no link to the internal edition.

## What a figure is

Every figure is the indicative monthly hosting cost to BES, at cost price: no margin, no tax, in US dollars, at the on-demand rates of BES's primary cloud region.
It covers hosting only; support, maintenance and upgrades are priced separately.
It is never presented as a price to the client.

- [ ] Every figure is labelled as an indicative monthly cost to BES, not a price.
- [ ] The monthly figure leads, with its annual equivalent beneath it.
- [ ] Displayed figures are rounded to a step set in the pricing configuration; the breakdown sums to the unrounded total.

A margin and a currency conversion are part of the configuration, applied to the displayed figures only.
While the margin is zero and the currency is US dollars, neither is shown.

## The cost model

A deployment's monthly cost is the sum of these lines.

- **Central server**, when the central server is in BES cloud: the server for the deployment's size band, its storage, and its snapshots.
  A central server the client hosts costs BES nothing.
- **Cloud facility servers**: one facility server, with its storage and snapshots, for each facility server in BES cloud.
- **Clone**: the copy of the deployment used to test upgrades, which always runs in BES cloud, even when the central server does not.
  Its storage is always paid for; its compute is paid for only while it runs.
  A standard clone is stopped by default and runs for a fixed share of the month; an always-on clone runs all month.
- **Platform share**: a fixed share of the BES infrastructure every deployment uses.
- **Backups share**: an equal share of the shared backups database, when BES takes backups.
- **Tupaia reporting**: a share of Tupaia and of the reporting pipeline, when the deployment feeds Tupaia.
- **Windows**: the extra compute cost of a Windows server, for each server in BES cloud that runs Windows, with a clone's share scaled by how long it runs.
  It applies to compute only; storage costs the same on either platform.

Backups and the clone are priced independently, so turning backups off removes only the backups share.

- [ ] The breakdown lists each line that applies with its monthly figure, and the total is their sum.
- [ ] The central server line appears only when the central server is in BES cloud.
- [ ] The clone is priced from the central server's size band even when the central server is client hosted.
- [ ] Turning backups off removes the backups share and leaves the clone as it was.
- [ ] A Windows server adds cost to compute only.
- [ ] Every input combination in the reference fixtures produces the fixture's lines, monthly and annual figures to the cent.

## The pricing configuration

Every number the model uses lives in one versioned file, `pricing.ron` at the repository root, beside the ruleset.
Nothing in the code or the interface carries a rate, a share, or a server size.

The size bands are one table: each band names the server it runs on, that server's hourly rate, its storage, and how many copies of that storage its snapshots hold.
The bands are the ones the ruleset derives, matched by name, and the table can hold a band the ruleset does not yet derive.
The facility server has its own row.

Each parameter carries the basis for its value and one of these confidence tags:

- **Decided**: settled by the person who owns the model.
- **Read**: read from a published rate or an observed bill.
- **Estimated**: an allocation of shared costs, expected to move.
- **Assumed**: a working assumption awaiting real figures.

- [ ] An assumptions panel lists every parameter with its value, tag and basis, so an estimate is never mistaken for a fact.
- [ ] A band missing from the table is reported as not priced rather than priced at another band.

## The calculator

The calculator prices inputs set directly, for costing a proposal or an existing deployment.
Its controls are:

- the size band, one of the configured bands;
- whether the central server is in BES cloud or client hosted;
- the number of facility servers in BES cloud, from none to twenty;
- the clone: off, standard or always on, and for a standard clone how much of the month it runs, starting at the configured share;
- whether BES takes backups, on by default;
- whether the deployment feeds Tupaia;
- for existing deployments still on Windows: a Windows central server, how many of the cloud facility servers run Windows, and a Windows clone.

- [ ] Changing any control updates the figure and breakdown straight away.
- [ ] The Windows central server control applies only while the central server is in BES cloud, and the Windows facility count never exceeds the cloud facility count.

## The priced wizard

The priced wizard is the wizard of [WIZ](../wizard/onboarding.md), with the same questions, flow and artifact, plus a running cost beside the questions that follows the answers.
The cost is read from the answers, with an assumed answer counting the same as a chosen one:

- the size band is the band the wizard derives; until it is derived, the cost reads as needing the sizing answers rather than pricing at the lightest band;
- the central server is in BES cloud when that is where the wizard puts it;
- backups are on unless BES may not take them;
- Tupaia is included only when it is asked for;
- a hosting region other than the primary one is priced at the primary region's rates, with a note that the difference is confirmed separately;
- BES-hosted integrations are not priced, since they are referred to a separate conversation.

The wizard asks where facility servers run, not how many there are, so the number of cloud facility servers starts as a guess from the facilities band and the share of facility servers in BES cloud, both set in the pricing configuration.
A control beside the cost overrides the guess, and the clone mode can be set there too, since neither is a client choice.
These overrides belong to the person pricing, not to the plan, and are not stored with it.

- [ ] The running cost updates as each answer changes.
- [ ] Before the deployment is sized, the cost asks for the sizing answers instead of showing a figure.
- [ ] With every facility server client hosted, the guessed cloud facility count is none.
- [ ] Overriding the cloud facility count or the clone mode changes the figure without changing the plan.
- [ ] A finalised plan opened in the priced edition shows its cost too.
- [ ] An answer left open that would change the cost, such as whether to feed Tupaia, is noted beside the figure.
