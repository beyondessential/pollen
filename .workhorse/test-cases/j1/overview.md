# Internal hosting cost: test cases

## Cost model

- [x] Every reference fixture (11 examples including a parameter override, 288 grid cases) matches lines, monthly and annual to the cent (verifies spec: COST)
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
- [x] Copy link in the priced edition copies the public address, never the internal one (verifies spec: COST)
- [ ] Making a new version or resuming from the priced edition stays in the priced edition (verifies spec: COST)
- [ ] The printed record from the priced edition carries no figure

## Displayed figures

- [x] Each line rounds to the configured step, the headline is the sum of the displayed lines and the annual figure is twelve times it, for every reference case (verifies spec: COST)
- [x] A cost quoted beside the breakdown, for an unsettled answer, is shown the same way as the lines (verifies spec: COST)
- [x] Lines that rest on estimated or assumed parameters carry that qualifier; zero lines are hidden
- [x] More facility servers than the model will price is refused

## Calculator

- [x] Defaults are backups on, smoke testing on a clone on, and Tupaia reporting on (verifies spec: COST)
- [x] Changing size, backups, smoke testing and central updates the figure and breakdown (verifies spec: COST)
- [x] Turning backups off removes only the backups share; turning smoke testing off removes the clone (verifies spec: COST)
- [x] Facility servers as BES cloud follow the estimator's facilities answer; client hosted prices none (verifies spec: COST)
- [x] A mix brings up the slider, starting at half the BES cloud count (verifies spec: COST)
- [x] The estimator sets the band the wizard would derive, bumps it for a connected system, and ignores "I'm not sure" (verifies spec: COST)
- [x] Choosing a band directly clears the estimator (verifies spec: COST)
- [ ] BES cloud facility servers with no estimate follow the band's typical count
- [x] Breakdown lines resting on estimated or assumed parameters carry their tag (verifies spec: COST)
- [x] The mix slider cannot pass the number of facility servers, and reads "N of total in BES cloud" (verifies spec: COST)

## Model v2.3

- [x] The pricing band ignores connected systems, though the ruleset's own size counts them (verifies spec: COST)
- [x] The estimator does not ask about connected systems
- [x] The version of the model shows with the line beneath the figure (verifies spec: COST)
- [x] A backups share that rounds to nothing is left out of the lines
- [ ] The catchment help text shows on the public wizard

## Priced wizard

- [x] Unsized plan asks for the sizing answers instead of a figure (verifies spec: COST)
- [x] All facility servers in BES cloud at 2–5 facilities guesses three; all client hosted guesses none (verifies spec: COST)
- [x] A mix shows the slider in place of the balance question, starting at half the guess (verifies spec: COST)
- [x] A slider setting is dropped when the facility count changes or the mix is left (verifies spec: COST)
- [x] Moving the slider or setting the clone moves the figure, not the plan; leaving the mix drops the slider's setting (verifies spec: COST)
- [x] Tupaia left open is noted and left out (verifies spec: COST)
- [ ] A finalised plan opened in the priced edition shows its cost (verifies spec: COST)
- [ ] Another AWS region shows the "confirmed separately" note (verifies spec: COST)
