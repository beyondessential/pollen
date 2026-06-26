# pollen

A guided configuration tool for standing up a new Tamanu deployment.

Pollen walks a BES team member and a prospective client through the technical
decisions a deployment requires, surfaces the consequences of leaving the
default supported path, and produces a permanent, shareable artifact that
records the choices for the client's IT team, the BES technical team, and as a
record of what was agreed.

It is public-facing and unauthenticated, and stores no client names, no free
text, and no sensitive data, so the artifacts it produces are non-sensitive and
addressable by an unguessable id alone.

For what the system does, see [`.workhorse/specs/wizard/onboarding.md`](.workhorse/specs/wizard/onboarding.md)
(the tool and its engine); the concrete v1 questions and consequences are
documented where the ruleset is authored. See [`docs/plans/`](docs/plans/) for
build plans.
