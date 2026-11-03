# Internal hosting cost: test cases

## Cost model

- [x] Every reference fixture (9 examples, 288 grid cases) matches lines, monthly and annual to the cent (verifies spec: COST)
- [x] Backups off removes only the backups share; the clone stays (verifies spec: COST)
- [x] A client-hosted Windows central, Windows facilities beyond the cloud count, and a Windows clone with the clone off add nothing (verifies spec: COST)
- [x] Clone uptime scales compute only; always-on equals 100%; out-of-range uptime is refused
- [x] Displayed figures round to the configured step (verifies spec: COST)
- [x] Every band the ruleset derives is priced
- [x] An unpriced band is refused rather than priced as another (verifies spec: COST)

## Access

- [x] Pricing endpoints answer 404 with no path or a wrong one (verifies spec: COST)
- [x] A wrong path in the browser shows "Not found." (verifies spec: COST)
- [x] The public wizard shows no figure (verifies spec: COST)
- [ ] Making a new version or resuming from the priced edition stays in the priced edition (verifies spec: COST)
- [ ] The printed record from the priced edition carries no figure

## Calculator

- [x] Changing size, backups and central updates the figure and breakdown (verifies spec: COST)
- [x] The Windows central control is disabled while the central server is client hosted (verifies spec: COST)
- [x] The assumptions panel lists parameters with their tags (verifies spec: COST)

## Priced wizard

- [x] Unsized plan asks for the sizing answers instead of a figure (verifies spec: COST)
- [x] All facility servers in BES cloud at 2–5 facilities guesses three; all client hosted guesses none (verifies spec: COST)
- [x] Overriding the facility count or clone moves the figure, not the plan (verifies spec: COST)
- [x] Tupaia left open is noted and left out (verifies spec: COST)
- [ ] A finalised plan opened in the priced edition shows its cost (verifies spec: COST)
- [ ] Another AWS region shows the "confirmed separately" note (verifies spec: COST)
