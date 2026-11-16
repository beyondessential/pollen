---
id: MIG
---

# Internal data migration effort

An internal estimator that sizes the work of moving a client's existing data into Tamanu: a timeline and a count of data person-days, each as a range.
It is separate from the hosting cost of [COST](../pricing/hosting-cost.md): it has its own model, its own configuration and its own address, and it produces no monetary figure.
The public tool described in [WIZ](../wizard/onboarding.md) shows no estimate, and nothing on it links to the estimator.

## Access

The estimator lives under a path segment set in the migration configuration, different from the one the hosting cost edition lives under.
The segment is memorable and not obvious.
It keeps the estimator from being found by accident; the figures are internal, not secret.
The estimator's endpoints answer only a request carrying the same segment, and answer anything else as though they did not exist.

- [ ] The estimator opens at the configured path.
- [ ] A wrong or missing segment shows the tool's ordinary "not found" page.
- [ ] The hosting cost edition's path does not open the estimator's endpoints, and the estimator's path does not open the hosting cost endpoints.
- [ ] The public wizard, its artifact, and its printed record carry no estimate and no link to the estimator.

## What an estimate is

An estimate is never a quote and never a promise of dates.
Every figure is a range.
Durations are elapsed time at a sustainable pace for a data team that also carries other work; the clock starts only once the prerequisites for the work are in place.
An estimate is worked out from the answers given and is not stored: the estimator takes no client name, no site name and no free text.

- [ ] No figure is shown as a single number where the model gives a range.
- [ ] Every estimate says its range is wide because data quality and shape vary widely between clients, and that a better estimate can be given after the initial analysis of the data (M0).
- [ ] The estimate names the version of the model its figures come from.
- [ ] Every answer can be left unknown, and each assumption made for an unknown answer is listed as a flag.
- [ ] Changing any answer updates the estimate straight away.

## Routes

A migration takes one of two routes, chosen first.

- **BES-led migration**: BES takes the data out of the client's system or files. Sized by tier and area, in weeks.
- **Client-prepared import**: the client formats the data into BES templates and does its own mapping, and the Tamanu forms and registries the data goes into already exist. Sized in data team days for each program.

## Scope of a BES-led migration

A migration is made of areas, each with a code:
initial analysis and set-up (M0), reference data (M1), users (M2), patients from basic registration to the full record (M3), encounters in full (M4) or as a readable history (M4h), medications (M5), medication administration (M6), labs (M7), imaging (M8), vaccinations (M9), clinical forms (M10f), disease or program forms (M10r), appointments with history (M11) or future only (M11f), billing as opening balances (M12b) or full history (M12f), documents (M13) and tasks (M14).

The scope starts from a tier, which names the areas it includes, and is then adjusted by the areas added to or removed from it.
When no tier is chosen, the scope starts from the tier the configuration names for that case, and the estimate says so.

The adjusted scope is completed by these rules, each of which is flagged when it changes the scope:

- M0 and M1 are always in scope, and M2 whenever anything beyond M0, M1 and M2 is.
- M6 brings in M5.
- Any area worked after encounters (M5 to M14) turns M4h into M4.
- Medications, medication administration, labs, imaging, vaccinations, clinical forms, billing and tasks sit on encounters in Tamanu, so they bring in M4; disease or program forms, appointments and documents do not.
- Encounters bring in patients.
- Future-only appointments take the place of appointments with history, and full billing history takes the place of opening balances, when both are in scope.
- A clinical or disease or program form count above zero brings its area in, and a count of zero or left blank takes it out: often there are no forms to migrate. Both form areas are optional; no tier includes them.

The patients area's range runs from basic registration at its low end to the full patient record at its high end, so how much of the record is moved is not asked.

- [ ] Each tier starts from its configured areas.
- [ ] Each completion rule above holds, and each is flagged when it changes the scope.
- [ ] M0 and M1 stay in scope when asked to be removed, with a flag.
- [ ] Forms are sized as their count times the range for one.
- [ ] A form count left blank is none, and the form area is listed as not included.
- [ ] Appointments and billing can each be left out of scope, as well as taken in either of their forms.

## Unanswered questions

An estimate runs from the least the migration could take to the most, and narrows as questions are answered.
Each unanswered question is answered with its cheapest answer to work out the low end, and with its costliest to work out the high end; each end is then worked out as below.
The cheapest and costliest answers are:

- **Tier:** reference data and users at the low end; at the high end, the full EHR with every other area, taking full encounters, appointments with history and full billing history.
- **Separate installs:** one at the low end, and the many band at the high end.
- **Documentation:** each answer yes at the low end and no at the high end. Earlier exports are priced only when the data dictionary is not yes.
- **The rest:** the answer that adds nothing at the low end, and the one that adds most at the high end.

Some questions take a default instead and are never priced both ways: sites going live (one), Tamanu already in use (no), and the form counts (none).

An area in scope at the high end only starts from nothing.
Workings that hold at both ends read as they are; the rest say which end they belong to.
The confidence and the key answers still unknown are worked out from the answers as given.

The estimate also lists the unanswered questions that move it, each with the range under its cheapest and its costliest answer, ordered by how far the two ranges are apart in months, then in person-days.

- [ ] With nothing answered, the range runs from the cheapest answers' low end to the costliest answers' high end.
- [ ] A question with a default is never listed among those that move the range.
- [ ] With every question answered, the range is the estimate as answered.
- [ ] Answering a question gives a range within the one before.
- [ ] The unanswered questions are listed, those that move the range most first.

## Ranges

Each area starts from its configured range of elapsed weeks; M0 takes a longer configured range for the full record tier.
The documentation answers then multiply the areas, and the other answers add their own configured elapsed weeks and data person-days, each a range or a single figure.
Yes, or the answer that costs least, changes nothing.

1. **Documentation.** A knowledgeable contact, exportable code lists and an up-to-date data dictionary each have a configured multiplier for partly and for no. Earlier exports with their queries are asked about only when the data dictionary is partly there or missing, and no exports have a multiplier of their own. Each multiplier applies to both ends of every area, set-up included, so documentation weighs in proportion to the scope; the multipliers compound. An unknown contact, code lists or data dictionary is taken as partly and flagged. The documentation is reported as good when the contact, code lists and dictionary are all yes, none when there is no knowledgeable contact, and partial otherwise.
2. **Source.** A source database that is not MySQL or Postgres adds its figures, except for a spreadsheet-only source. Several systems, or a mix, add theirs. A spreadsheet-only source adds nothing, but marks the whole estimate as an upper bound and flags that the template-based route should be estimated first; several systems or a mix flag that a template-based route may suit part of the source. Separate installs of the source system (the same system at another site, keeping its own data with no central database) are read in three configured bands: one adds nothing, and a few and many each add their figures.
3. **Size.** The patients seen a day at the busiest site, and, when visit history (M4 or M4h) is in scope, the years of history to migrate, are each answered in one of three configured bands. Each band has a configured multiplier; the patients a day's applies first, then the years', and together they multiply both ends of M1 and of every area from M3 onwards. An unknown answer applies no multiplier and is flagged.
4. **Staff to be migrated.** Past staff count when records are to be attributed to them. Up to the configured number, and over it, each set M2 to a configured number of weeks; unknown, M2 keeps its full range.
5. **Sites.** The first site's cutover is part of the timeline. Each site beyond it adds its configured figures. A number of sites left blank is one, and flagged.
6. **Tamanu already in use.** Checking for conflicts with records entered since go-live adds its configured figures. Left blank, it is taken as not in use.
7. **Reviews.** A client slow to review adds configured weeks to the high end of the timeline only, and no person-days.

Duplicate patient records are not estimated; finding and merging them is listed as not included.

- [ ] Each adjustment applied is listed in the estimate's workings, one line each, with the weeks and person-days it adds.
- [ ] Each area in the estimate shows its adjusted range in weeks, its data person-days, and the basis its starting range rests on, so a derived figure is not mistaken for a calibrated one.

## Timeline

The timeline is the sum of:

- the longer of M0 and M1, since reference data and users run during M0;
- the patients area, then the encounters area;
- the areas after encounters, added up and shared between the data people, but never shorter than the longest single one of them other than forms and registries, which can be split between people;
- a configured time for the first site's cutover;
- the weeks each answer adds, as above;
- for a client slow to review, configured weeks on the high end only, after the rest are added up.

Each of these is a range, and they are added up as independent estimates: each is spread around its midpoint, the total runs from the sum of the midpoints less the root of the sum of the squared half-spreads to the sum plus it, since independent areas rarely all run long or all run short together.
The areas after encounters are added up the same way before being shared.
Data person-days are added up the same way.
(The configuration can instead add every low up and every high up.)

It is given in months, to the nearest half month, for one, two and three data people, marking the staffing that was answered, or two when it was not.
When the timeline is the same for all three, it is given once, saying that extra data people do not shorten this scope.

- [ ] The timeline follows the rule above for one, two and three data people.
- [ ] Only the areas after encounters are shared between the data people; nothing else runs faster with more people.
- [ ] Data person-days grow by a configured share for each data person beyond the first, compounding, for keeping the work aligned.

## Person-days

Data person-days for an area are its elapsed weeks times the working days in a week times the share of that time the data team spends on it, with one share for M0 and another for every other area.
The first site's cutover adds a configured range, and each answer its configured person-days.
A review allowance is added to the data total, and then the share for each data person beyond the first.

For every tier but the full record, M0 and M1 are reported on their own, without the review allowance, as scoping and reference data that may fall largely to the Program Manager; the rest is the data team's.
For the full record tier there is one data figure.
Totals are rounded to the nearest five, or to the nearest whole day below thirty.

- [ ] Each answer's person-days reach the total, with the review allowance and the share for extra data people.

## Confidence

Confidence says how much is known about the migration, not how hard it is.

The key answers are the tier, the areas when no tier is chosen, the kind of source, the number of source instances, the source database (unless the source is spreadsheets only), the contact, code lists and data dictionary, earlier exports when the data dictionary is not yes, the patients a day, the years of history when visit history is in scope, the staff to be migrated, and review speed.

- **Low** when the tier is not chosen, the kind of source is unknown, there is no knowledgeable contact, the patients a day are unknown, or the configured number of key answers or more are unknown.
- **High** only when documentation is rated good, no more than the configured number of key answers are unknown, and no area in scope rests on a derived range.
- **Medium** otherwise.

- [ ] The estimate shows its confidence and lists the key answers still unknown.

When documentation is partial, the estimate also shows the timeline and person-days if documentation turns out good, and if there turns out to be no knowledgeable contact, so the value of the client's documentation is visible.

- [ ] A partial documentation rating shows both alternatives; any other rating shows neither.

## Flags

Two conditions are raised ahead of the other flags, for the Data Lead to weigh before the estimate goes further: Tamanu already being in use at the site, and there being neither a knowledgeable contact nor a data dictionary.
Neither stops the estimate.

Every estimate lists what it does not include: post go-live support, with its configured range for each cutover; preparing data for Tamanu upgrades during the migration; FHIR integration on the deployment; finding and merging duplicate patient records; each form area with none counted; and, when forms are in scope, building any that do not exist in Tamanu yet.
Project management, infrastructure and development involvement are not estimated.

- [ ] The two conditions above appear first among the flags when they hold.
- [ ] The not-included list appears on every estimate.

## Client-prepared imports

A program is the data for one health program that the client formats and hands over for import, such as NCD or antenatal care at the sites that run it.
Data collected with the same tools and handed over together belongs in one program.
A program usually has one registry, which holds its patients, and several forms, each with its own template and import.
An engagement is a list of programs, each with a number of sites, whether it has a registry, a number of forms, and a rough number of cases.
A program's data team days are, as a range:

- for each form, a range set by the program's rough case numbers, which fall in one of three configured bands;
- a fixed amount for the registry when the program has one, counted once whatever the number of sites;
- for each site beyond the first, a configured range for variations in how it was implemented;
- a contingency;
- a standing buffer.

Creating the import templates is added once for the engagement unless they exist; existing templates add nothing.
Unknown answers are priced at both ends and flagged: the low end takes the templates as existing, no registry, one site and the bottom band of case numbers; the high end takes the templates as not existing, a registry, the configured most sites and the top band.
Patients come from the registry and are included in its import, or in the first form's when there is no registry.

Each program's elapsed time is its data team days divided by the configured days worked on it in a week, counted from when the client's formatted data arrives; programs can run in parallel.

The estimate lists what it does not include: the client's time formatting the data; building or changing Tamanu forms and registries; reports and DHIS2 set-up; post go-live support; and removing and re-importing data if the client's data changes after import.

- [ ] Each program shows its range of data team days, and the total is the sum of the programs and the templates.
- [ ] Days are shown to the nearest half day.
- [ ] A program's days are the sum of the parts above, at each end of the range.

## The migration configuration

Every number the model uses lives in one versioned file, `data-migration.ron` at the repository root, beside the ruleset and the pricing configuration.
Nothing in the code or the interface carries a range, a multiplier, a share or a threshold.
The file names each area, its range and the basis for it, the areas each tier includes, and the wording each area and tier is offered under.

Each area's range carries one of these bases:

- **Calibrated**: taken from a previous BES-led migration.
- **Team experience**: set from the team's experience.
- **Derived**: worked out from another area's range.

A set of reference cases records what the estimator gives for each of their inputs, for both routes, so a change to the model is never made by accident.
They are recorded from the estimator itself, and recorded again after each deliberate change, with the difference reviewed.

- [ ] Every reference case gives what was last recorded for it.
- [ ] A configuration that leaves out an area or a tier, or lists one twice, is rejected.
- [ ] A configuration whose path segment matches the hosting cost edition's is rejected.
