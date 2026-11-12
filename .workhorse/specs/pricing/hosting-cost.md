---
id: COST
---

# Internal hosting cost

An internal edition of the tool that shows what a deployment costs BES to host each month.
It has two surfaces sharing one cost model: a calculator that prices a handful of inputs directly, and a priced edition of the wizard that keeps a running cost beside the questions.
The public tool described in [WIZ](../wizard/onboarding.md) shows no figures, and nothing on it links to the internal edition.

## Access

The internal edition lives under a path segment set in the pricing configuration, never in the ruleset.
The segment is memorable and not obvious.
It keeps the edition from being found by accident, the way an artifact's identifier keeps the artifact; the figures are internal, not secret.
The cost endpoints answer only a request carrying the same segment, and answer anything else as though they did not exist, so the public tool cannot reach a figure.

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
- [ ] Each line is displayed rounded to a step set in the pricing configuration, the monthly figure is the sum of the displayed lines, and the annual figure is twelve times it, so what is shown adds up.
- [ ] A figure quoted anywhere beside the breakdown, such as a cost still depending on an unsettled answer, is displayed the same way as the lines.
- [ ] The line beneath the figure says it is USD at cost and not a price to the client, and says instead that it includes the margin once a margin is applied.
- [ ] The same line names the version of the model the figures come from.
- [ ] A line that rounds to nothing is left out.

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
- **Backups and clone feeding**: a share of the shared data cluster that takes backups and feeds clones, by the deployment's database size, when BES takes backups.
- **Backup storage**: the share of the backup buckets, by the deployment's database size, when BES takes backups.
- **Demo/training environment**: a demo or training copy of the deployment on the shared demo cluster, an even share of the cluster across its environments, the same whatever the size band, the central server, the clone or backups, on unless declined.
- **Tupaia reporting**: over and above backups, a flat share of Tupaia's compute and the reporting pipeline plus the reporting part of the data cluster by database size, when the deployment has Tupaia reporting.
- **Windows**: the extra compute cost of a Windows server, for each server in BES cloud that runs Windows, with a clone's share scaled by how long it runs.

The database size is estimated from the size band and is never entered.
  It applies to compute only; storage costs the same on either platform.

Backups and the clone are priced independently, so turning backups off removes only the backups and clone feeding share.

- [ ] The breakdown lists each line that applies with its monthly figure, and the total is their sum.
- [ ] The central server line appears only when the central server is in BES cloud.
- [ ] The clone is priced from the central server's size band even when the central server is client hosted.
- [ ] Turning backups off removes the backups and clone feeding share and leaves the clone as it was.
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

- [ ] A breakdown line that rests on an estimated or assumed parameter carries that tag, so an estimate is never mistaken for a fact.
- [ ] A band missing from the table is reported as not priced rather than priced at another band.

## The calculator

The calculator prices inputs set directly, for costing a proposal.
Its controls are:

- the size band, one of the configured bands, with an estimator that works the band out from the sizing questions the wizard asks (the catchment, the facilities and the mobile users), using the wizard's own derivation of size;
- whether the central server is in BES cloud or client hosted;
- where the facility servers run: BES cloud, client hosted, or a mix, where a mix brings up a slider that splits the deployment's facility servers between client hosted and BES cloud;
- whether BES takes backups, on by default;
- whether upgrade smoke testing runs on a clone, on by default, priced as a standard clone at the configured share of the month;
- whether the deployment has a demo/training environment, on by default;
- whether the deployment has Tupaia reporting, on by default.

The deployment's facility servers are counted from the facilities answer of the estimator, or, when the size band was chosen directly, from a typical number for that band.
A mix starts with a configured share of them in BES cloud, and the slider runs from none in BES cloud to all of them, labelled at each end and reading how many of the total are in BES cloud.

- [ ] Changing any control updates the figure and breakdown straight away.
- [ ] Choosing the facility servers as BES cloud prices a count that follows the facilities estimate, or the size band when nothing has been estimated.
- [ ] Choosing client hosted prices no facility servers.
- [ ] Choosing a mix shows the slider, starting at the configured share of the facility servers.
- [ ] The slider cannot put more facility servers in BES cloud than there are, and returns to its starting share when the size or the estimate changes.
- [ ] The estimator sets the size band to the band the wizard would derive from the same answers.
- [ ] Choosing a size band directly clears the estimator's answers.
- [ ] Turning backups off removes only the backups and clone feeding share.
- [ ] Turning off upgrade smoke testing removes the clone.
- [ ] Turning off the demo/training environment removes only its line, and the line is the same for every size band.
- [ ] The breakdown lists backup storage straight after backups and clone feeding, and the demo/training environment straight after that, before Tupaia reporting.

## The priced wizard

The priced wizard is the wizard of [WIZ](../wizard/onboarding.md), with the same questions, flow and artifact, plus a running cost beside the questions that follows the answers.
The cost is read from the answers, with an assumed answer counting the same as a chosen one:

- the size band is the highest band the catchment, facilities and mobile answers reach, without the extra band the wizard adds for connected systems, which do not size a deployment for costing; until it is derived, the cost reads as needing the sizing answers rather than pricing at the lightest band;
- the central server is in BES cloud when that is where the wizard puts it;
- backups are on unless BES may not take them;
- the demo/training environment is on unless the person pricing switches it off beside the cost;
- Tupaia reporting is included unless it is declined;
- a hosting region other than the primary one is priced at the primary region's rates, with a note that the difference is confirmed separately;
- BES-hosted integrations are not priced, since they are referred to a separate conversation.

The wizard asks where facility servers run, not how many there are, so the number of cloud facility servers is a guess from the facilities band and the share of facility servers in BES cloud, both set in the pricing configuration.
When facility servers are a mix of both, a slider that splits them between client hosted and BES cloud takes the place of the question about the balance, starting at the guess and running up to all of them.
The clone mode can be set beside the cost, since it is not a client choice.
These settings belong to the person pricing, not to the plan, and are not stored with it.

- [ ] The running cost updates as each answer changes.
- [ ] Before the deployment is sized, the cost asks for the sizing answers instead of showing a figure.
- [ ] With every facility server client hosted, the cloud facility count is none.
- [ ] With every facility server in BES cloud, the cloud facility count is the guess for the facilities band.
- [ ] With a mix, the slider appears where the balance question would, and moving it changes the figure without changing the plan.
- [ ] Setting the clone mode changes the figure without changing the plan.
- [ ] A finalised plan opened in the priced edition shows its cost too.
- [ ] An answer left open that would change the cost, such as whether to feed Tupaia, is noted beside the figure.
