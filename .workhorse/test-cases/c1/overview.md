# Simplify the estimate questionnaire for non-technical users

Coverage this card owes. A ticked box is covered by an automated test that has
been run; an unticked one is a scenario still owed, not one decided against.

## Answering, assuming, declining (verifies spec: WIZ)

- [x] A plan with nothing answered reports only catchment and facilities as required.
- [x] Answering those two alone is enough to finalise, with the technical section never opened.
- [x] Every question left blank either takes its assumed answer or records as open, never silently vanishes.
- [x] An assumed answer triggers the same consequences a chosen one would.
- [x] Assuming an answer that reveals a further assumable question resolves both, not just the first.
- [x] A question hidden again by a later assumption is not reported as assumed.
- [x] Declining a question fires neither the affirmative nor the negative consequence.
- [x] Declining a question hides the questions it gates rather than asking them against an unknown.
- [x] An "I'm not sure" band does not size the deployment, and does not read as the highest band.
- [x] An unsure multi-select is recorded as open and asserts nothing.
- [x] An unsure mobile count does not trigger the rules that assume mobile clients are in play.
- [x] DNS ownership is never assumed, because it varies too much between clients.

## Hosting shape (verifies spec: WIZ)

- [x] All BES cloud asks nothing further about Iti, operating system, or provisioning.
- [x] A mix reveals the balance question; the all-one answers do not.
- [x] The Iti question appears only once something sits outside BES cloud.
- [x] Every non-cloud site on a mini-server removes the operating system and provisioning questions.
- [x] Those sites still raise the client network requirements.
- [x] Central hosted by the client reads as off the standard path.

## Consequences

- [x] The untouched default path raises no callout at all.
- [x] Short backup retention reads as a recovery tradeoff, not a neutral preference.
- [x] Upgrading less often than the ten-release support window is flagged; every two months is not.
- [x] AMD64 carries a support and cost penalty; ARM64 does not.
- [x] Sydney is the only named region, and is the assumed one.
- [x] Declining telemetry conflicts with dashboards and with mobile clients.
- [x] A blocking conflict still produces a full artifact rather than refusing to record.

## Lifecycle and robustness (verifies spec: WIZ)

- [x] Finalising is refused only while a required question is unanswered.
- [x] A plan finalised with open questions finalises successfully and keeps them recorded.
- [x] A stored ruleset the engine cannot read reports a conflict rather than an internal error.
- [x] The bundled ruleset parses, validates, and leaves every question resolvable without an answer.
- [x] Forking an interim artifact carries its open questions into the new draft for settling.
- [ ] A plan bound to the previous ruleset still loads after a ruleset change, and offers the update.

## Interface (manual)

- [ ] The technical section arrives collapsed, and reads as an invitation to sharpen the estimate.
- [ ] The rail shows only the size until something goes off the standard path.
- [ ] No verdict banner appears on a plan with nothing wrong.
- [ ] An assumed answer is indistinguishable from a chosen one in the form.
- [ ] Choosing a non-standard option surfaces its warning at the point of choice.
- [ ] The finalise button reads as an interim plan while questions are open.
- [ ] An interim artifact shows its badge and lists open questions ahead of the detail.
- [ ] Assumptions are listed on the artifact and are correctable through a new version.
- [ ] The PDF export places open questions directly after the verdict, and prints no buttons.
- [ ] Printing with a group collapsed still carries that group's items, by the button and by the browser's own print command.
- [ ] Audience groups arrive open, and collapsing one leaves the others alone.
- [ ] Only the client's required actions carry a tick box; acknowledgements and advisories carry none.
- [ ] A tick survives a reload of the same artifact, and does not appear on a different one.
- [ ] Tick state never reaches the server, and the printed record shows no boxes.
- [ ] The viability callout expands to name the choices behind its count, and prints expanded.
- [ ] Finalising lands the reader at the top of the artifact, and a link naming a section still reaches it.
- [ ] Search appears only on an artifact long enough to need it.
- [ ] The balance question renders as one row of three, matching the sizing bands.
