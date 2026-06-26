---
id: WIZR
---

# Onboarding wizard ruleset (v1)

The v1 ruleset the engine evaluates ([WIZ](onboarding.md)).
It defines the decision points the tool captures, the order it captures them in, and the consequences each triggers.

The exact option lists, band thresholds, size mappings, cost tiers, and consequence wording are data confirmed with BES.
What this spec fixes is the set of decision points v1 covers and the cross-field relationships between them.

## Question flow

The flow follows the natural cascade of the decisions, and earlier intents constrain later questions.

1. **Analytics intent.**
   Whether the deployment connects to the BES analytics platform.
   Asked first; it constrains the backup decisions, because connecting means BES processes the data, which requires at minimum a low-retention backup.
2. **Integrations.**
   Which integration categories are wanted, if any (laboratory, pharmacy/warehousing, and room for more), each optionally a named system.
   The presence of any integration raises a planning advisory that the central server is sized up; the count and mix do not scale this.
   A request for BES to host an integration raises a referral.
3. **Sizing.**
   Catchment population, facility count, and mobile client count, each chosen as a band.
   The highest band reached derives the deployment size, which drives server staging.
4. **Topology.**
   Where the central server runs (BES cloud, customer cloud treated as on-premises, or true on-premises), and the mix of facility placement classes present (BES cloud, on-premises bare-metal, on-premises virtualized, or a BES-built appliance).
   The presence of each class unlocks the relevant sections below.
   BES cloud means BES-managed hosting where the client gets application access by default and system access only on request.
5. **Region** — only when any part of the deployment is in BES cloud.
   A default region, an alternate region, or another region; another region carries a cost consequence.
6. **Platform and operating system** — shown per class present.
   BES cloud and the appliance are fixed and informational.
   On-premises offers a preferred architecture, a fully-supported alternative, a phased-out option that carries cost, operational, and support consequences, and an unsupported option that is blocking — the product is open source and the client can run it, but BES cannot access or support it, which removes support, backups, analytics, and clone-upgrade testing.
   On a supported platform, the client is required to use the BES-provided server image set up the BES way.
7. **On-premises server detail** — only when any on-premises facility is present.
   Whether servers are bare-metal (resources hard to change, so provision generously up front) or virtualized (resources can scale, so start small and grow, with a recommendation to take VM-level backups).
8. **Backups** — two distinct dimensions.
   First, capability: whether BES is allowed to take backups at all.
   Second, retention (only when capability is allowed): full, low-retention (process-only), or none (self-managed).
   The two are separate because clone-upgrade testing and analytics need only the *ability* to take a backup, even with near-zero retention.
   Disallowing backups entirely is blocking when analytics or BES-managed recovery is also wanted, and otherwise a non-default acknowledgment that removes recovery and clone-upgrade testing.
   Self-managed retention is a non-default acknowledgment on its own; the client must re-assure that they run their own backups.
9. **Upgrade cadence** — advisory only, never a contractual term.
   How often the client intends to upgrade.
   A longer cadence raises an advisory: BES releases frequently, so larger jumps mean heavier migrations, more potential downtime, and stronger reliance on clone-upgrade testing — which in turn needs backup capability.
10. **Networking** — a mix of always-present requirements and triggered ones.
    DNS authority (BES controls DNS, or the client does and must then support automatic certificate issuance and a domain-level wildcard); remote access for managed servers (the BES remote-access network is the requirement, anything else is an exceptional, costed special arrangement); time synchronization (an internal time server or outbound access to public time pools); a partition-resilience recommendation when on-premises facilities are present; and a request for facility egress ranges when the central server is in BES cloud and on-premises facilities are present.

## Forward guidance and visibility

The flow hides questions that do not apply and warns forward when an earlier answer constrains a later one.

- The region question appears only when any BES cloud is present.
- The on-premises detail, partition-resilience, and egress-range items appear only when on-premises facilities are present.
- The retention question appears only when backup capability is allowed.
- An analytics intent constrains the backup questions: the tool warns, before the user reaches backups, that backups cannot be fully disabled, and the final consistency check still catches an incompatible combination if the user pushes through.
