# pollen

A guided configuration tool for standing up a new Tamanu deployment.

Pollen walks a BES team member and a prospective client through the technical
decisions a deployment requires, surfaces the consequences of leaving the
default supported path, and produces a permanent, shareable artifact that
records the choices for the client's IT team, the BES technical team, and as a
record of what was agreed.

It is a public-facing, unauthenticated companion to the Tamanu fleet —
deliberately isolated from the operator control plane: it stores no fleet data,
no client names, and no free text, so the artifacts it produces are
non-sensitive and addressable by an unguessable id alone.

See [`.workhorse/specs/wizard/onboarding.md`](.workhorse/specs/wizard/onboarding.md)
for what the system does, and [`docs/plans/`](docs/plans/) for build plans.
