import type { Page } from "@playwright/test";

import { PRICING_PATH } from "./fixture";
import { expect, test } from "./test-fixtures";

const BASE = `/${PRICING_PATH}`;

async function answer(page: Page, question: string, option: string) {
	const card = page.locator(".card", { hasText: question });
	const patched = page.waitForResponse((r) => r.url().includes("/api/applications/patch"));
	await card.getByRole("button", { name: option }).first().click();
	await patched;
}

test("the calculator prices its inputs and follows each control", async ({ page }) => {
	await page.goto(BASE);
	await expect(page.getByRole("heading", { name: "Hosting cost calculator" })).toBeVisible();

	const figure = page.locator(".calc-result .cost-v");
	const lines = page.locator(".calc-result .cost-lines");
	const group = (name: string) => page.getByRole("radiogroup", { name, exact: true });

	// Tiny, BES cloud central, client-hosted facilities, backups, smoke testing
	// on a clone and Tupaia reporting, all on by default: 637.33. Each line
	// rounds to $5 (80, 85, 40, 40, 395) and the headline is their sum.
	await expect(figure).toHaveText("$640");
	await expect(page.locator(".calc-result .cost-annual")).toHaveText("$7,680 a year");
	await expect(lines).not.toContainText("Cloud facility servers");
	await expect(lines).toContainText("Tupaia reporting");
	await expect(lines.locator("li", { hasText: "Platform share" })).toContainText("Estimate");
	await expect(lines.locator("li", { hasText: "Backups share" })).toContainText("Assumed");

	await page.getByRole("radio", { name: "Medium" }).click();
	await group("Tupaia reporting").getByRole("radio", { name: "No" }).click();
	// 544.59.
	await expect(figure).toHaveText("$545");

	// Backups off removes only the backups share; the clone stays.
	await group("BES takes backups").getByRole("radio", { name: "No" }).click();
	await expect(figure).toHaveText("$505");
	await expect(lines).toContainText("Clone");
	await expect(lines).not.toContainText("Backups share");

	// Without smoke testing there is no clone.
	await group("Upgrade smoke testing on a clone").getByRole("radio", { name: "No" }).click();
	await expect(lines).not.toContainText("Clone");

	// A client-hosted central drops the central server.
	await group("Central server").getByRole("radio", { name: "Client hosted" }).click();
	await expect(lines).not.toContainText("Central server");

	// Estimates are tagged where they appear; there is no list of assumptions.
	await expect(page.getByRole("button", { name: "Assumptions" })).toHaveCount(0);
});

test("the calculator asks where facility servers run, and estimates the size", async ({ page }) => {
	await page.goto(BASE);
	const figure = page.locator(".calc-result .cost-v");
	const lines = page.locator(".calc-result .cost-lines");
	const group = (name: string) => page.getByRole("radiogroup", { name, exact: true });
	await group("Tupaia reporting").getByRole("radio", { name: "No" }).click();

	// Only a mix brings up the slider.
	const slider = page.getByRole("slider", { name: "Facility servers in BES cloud" });
	await expect(slider).toHaveCount(0);

	// The estimator sets the band, and the facilities answer sizes the count.
	await page.getByRole("button", { name: "Estimate size" }).click();
	const facilities = page.getByRole("group", { name: "How many facilities will use Tamanu?" });
	await facilities.getByRole("button", { name: "6 - 20" }).click();
	await expect(page.getByRole("radio", { name: "Medium" })).toBeChecked();
	await group("Facility servers").getByRole("radio", { name: "BES cloud" }).click();
	await expect(lines).toContainText("10 × t4g.medium");
	await expect(slider).toHaveCount(0);

	// A mix starts at half of that, and the slider moves it.
	await group("Facility servers").getByRole("radio", { name: "A mix" }).click();
	await expect(slider).toHaveValue("5");
	await expect(page.locator(".calc-field .slider")).toContainText("Client hosted");
	await expect(page.locator(".calc-field .slider")).toContainText("5 of 10 in BES cloud");
	await expect(page.locator(".calc-stack .cost-hint")).toHaveText("Assumes 10 facility servers, from the estimate.");
	// The slider splits the facility servers there are: it cannot pass them.
	await expect(slider).toHaveAttribute("max", "10");
	await expect(lines).toContainText("5 × t4g.medium");
	await slider.fill("2");
	await expect(lines).toContainText("2 × t4g.medium");

	// Client hosted prices none.
	await group("Facility servers").getByRole("radio", { name: "Client hosted" }).click();
	await expect(lines).not.toContainText("Cloud facility servers");

	// A connected system bumps the estimate one band; choosing a band directly
	// clears the estimator.
	const systems = page.getByRole("group", { name: "Does Tamanu need to connect to other systems?" });
	await expect(systems).toContainText("e.g. SENAITE, mSupply, DHIS2");
	await systems.getByRole("button", { name: "Yes" }).click();
	await expect(page.getByRole("radio", { name: "Large" })).toBeChecked();
	await page.getByRole("radio", { name: "Small" }).click();
	await expect(facilities.getByRole("button", { name: "6 - 20" })).toHaveAttribute("aria-pressed", "false");
	await expect(figure).not.toHaveText("Not priced");
});

test("a priced plan keeps a running cost beside the questions", async ({ page }) => {
	await page.goto(`${BASE}/new`);
	await expect(page).toHaveURL(new RegExp(`${BASE}/a/`));
	const rail = page.locator(".rail");
	await expect(rail.locator(".cost-v")).toHaveText("Not priced");
	await expect(rail).toContainText("Answer the first two questions to see a cost.");
	// Nothing to adjust until there is a figure.
	await expect(rail.getByRole("radiogroup", { name: "Upgrade smoke testing on a clone" })).toHaveCount(0);

	await answer(page, "How many people does the health system serve?", "Under 1,000");
	await answer(page, "How many facilities will use Tamanu?", "One");
	await answer(page, "Do you want dashboards and reporting?", "No, Tamanu only");
	// Tiny, BES cloud central, client-hosted facilities, standard clone, backups.
	await expect(rail.locator(".cost-v")).toHaveText("$245");

	// No slider until facility servers are a mix; the balance question is
	// replaced by it, never shown alongside.
	const slider = page.getByRole("slider", { name: "Facility servers in BES cloud" });
	await expect(slider).toHaveCount(0);
	await answer(page, "Where should the facility servers run?", "A mix of both");
	await expect(page.getByRole("heading", { name: "Roughly what balance?" })).toHaveCount(0);
	await expect(
		page.getByRole("heading", { name: "How many facility servers run in BES cloud?" }),
	).toBeVisible();
	// One facility, half of it in BES cloud: one server. 311.80, shown as the
	// lines add up (80, 70, 85, 40, 40).
	await expect(slider).toHaveValue("1");
	await expect(rail.locator(".cost-v")).toHaveText("$315");

	// Settings belong to the pricer: they move the figure, not the plan.
	await expect(page.locator(".slider")).toContainText("1 of 1 in BES cloud");
	await slider.fill("0");
	await expect(rail.locator(".cost-v")).toHaveText("$245");
	await slider.fill("1");
	await expect(rail.locator(".cost-v")).toHaveText("$315");
	await expect(page.getByRole("button", { name: "Use 1" })).toBeVisible();
	await rail
		.getByRole("radiogroup", { name: "Upgrade smoke testing on a clone" })
		.getByRole("radio", { name: "No" })
		.click();
	await expect(rail.locator(".cost-v")).toHaveText("$230");

	// A setting is let go when the facility count it was made for changes.
	await slider.fill("0");
	await answer(page, "How many facilities", "2 - 5");
	await expect(slider).toHaveAttribute("max", "3");
	await expect(slider).toHaveValue("2");
	await expect(page.getByRole("button", { name: /^Use \d/ })).toHaveCount(0);

	// Leaving the mix takes the slider, and its setting, out of the figure.
	await answer(page, "Where should the facility servers run?", "All client hosted");
	await expect(slider).toHaveCount(0);
	await expect(rail.locator(".cost-v")).toHaveText("$160");
});

test("a wrong path is not found, and the public wizard shows no figure", async ({ page }) => {
	await page.goto("/not-the-pricing-path-at-all");
	await expect(page.getByText("Not found.")).toBeVisible();

	await page.goto("/");
	await expect(page).toHaveURL(/\/a\//);
	await answer(page, "How many people does the health system serve?", "Under 1,000");
	await answer(page, "How many facilities will use Tamanu?", "One");
	await expect(page.locator(".rail")).not.toContainText("$");
	await expect(page.locator(".cost")).toHaveCount(0);
});

test("copy link on a plan never carries the internal address", async ({ page, context }) => {
	await context.grantPermissions(["clipboard-read", "clipboard-write"]);
	await page.goto(`${BASE}/new`);
	await expect(page).toHaveURL(new RegExp(`${BASE}/a/`));
	const id = page.url().split("/a/")[1];
	await answer(page, "How many people does the health system serve?", "Under 1,000");
	await answer(page, "How many facilities will use Tamanu?", "One");
	await page.getByRole("button", { name: /Finalise/ }).click();
	await page.getByRole("button", { name: "Copy link" }).click();
	const copied = await page.evaluate(() => navigator.clipboard.readText());
	expect(copied).toContain(`/a/${id}`);
	expect(copied).not.toContain(PRICING_PATH);
});
