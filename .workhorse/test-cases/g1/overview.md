# G1 · Compute requirements in results

Scenarios verifying the artifact presents compute requirements for the classes a
deployment actually uses, drawn from the ruleset (verifies spec: WIZ, Compute
requirements).

## Engine: which classes surface

- [x] Default path (BES-cloud Central, all-client facilities, no mobile) surfaces facility server and user devices, but not Central, Iti, or mobile
- [x] A fully BES-cloud deployment with no mobile surfaces only user devices
- [x] Client-hosted Central surfaces the Central server requirement
- [x] Mobile users surface the mobile device requirement; an unsure mobile count surfaces nothing
- [x] Some sites on Iti keep the facility-server requirement and add the Iti one; every site on Iti drops the facility server and keeps only Iti
- [x] Every requirement profile in the ruleset names a class and has at least one spec row

## Artifact rendering

- [x] The finalised web view shows a "Compute requirements" section below the consequence groups, one block per present class, each with its spec rows
- [ ] Each block shows the who-provisions summary and, where authored, the note
- [x] The section is absent from no deployment (user devices always present, so it never renders empty)
- [ ] The PDF export includes the compute requirements with every block expanded

## Ruleset integrity

- [x] The bundled ruleset with requirements parses, validates (unique requirement ids), and hashes deterministically
- [x] An older artifact bound to a ruleset without a `requirements` block still loads (serde default)
