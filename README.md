# pollen

A guided walkthrough of the technical details to specify, understand, and
acknowledge before signing up a new Tamanu deployment.

Pollen walks a BES team member and a prospective client through the technical
decisions a deployment requires, surfaces the consequences of leaving the
default supported path, and produces a permanent, shareable artifact that
records the choices for the client's IT team, the BES technical team, and as a
record of what was agreed.

It is public-facing and unauthenticated, and stores no client names, no free
text, and no sensitive data, so the artifacts it produces are non-sensitive and
addressable by an unguessable id alone.

## Ruleset

The ruleset.ron file describes all the content of the form and the final report.
It is pulled from `main` by the live site every five minutes.

You can preview a ruleset change by pushing it to a branch in this repo and
appending `?config=name-of-branch` to live URLs. Doing so with an in-progress
plan will fork that plan to a new version with the config being previewed.

Changing the _engine_ or _visual style_ requires code changes and a full deploy.

## Internal models

Two internal tools sit beside the public wizard, each at its own path and each
driven by one RON file at the repository root:

- `pricing.ron`: the hosting cost model, a calculator and a priced edition of
  the wizard (spec `COST`).
- `data-migration.ron`: the data migration effort model, an estimator that sizes
  moving a client's existing data into Tamanu as a timeline and data person-days
  (spec `MIG`). It is separate from the hosting cost and carries no dollar figure.

Both are bundled at build time, so changing either needs a deploy.

## Local dev

Run `just watch-api` and `just watch-web` in two terminals and open <http://localhost:8090>.
