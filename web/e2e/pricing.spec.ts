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
	// on a clone and Tupaia reporting, all on by default: 637.33.
	await expect(figure).toHaveText("$635");
	await expect(page.locator(".calc-result .cost-annual")).toHaveText("$7,650 a year");
	await expect(lines).toContainText("Tupaia reporting");

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

	await page.getByRole("button", { name: "Assumptions" }).click();
	await expect(page.getByText("Standard clone uptime")).toBeVisible();
	await expect(page.locator(".conf-estimated").first()).toBeVisible();
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
	await expect(lines).toContainText("5 × t4g.medium");
	await slider.fill("2");
	await expect(lines).toContainText("2 × t4g.medium");

	// Client hosted prices none.
	await group("Facility servers").getByRole("radio", { name: "Client hosted" }).click();
	await expect(lines).not.toContainText("t4g.medium (2 vCPU, 4 GB), 200 GB");

	// A connected system bumps the estimate one band; choosing a band directly
	// clears the estimator.
	const systems = page.getByRole("group", { name: "Connected systems" });
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
	await expect(rail).toContainText("Needs the catchment and facilities answers.");

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
	await expect(page.getByRole("heading", { name: "How many facility servers will BES host?" })).toBeVisible();
	// One facility, half of it in BES cloud: one server. 311.80.
	await expect(slider).toHaveValue("1");
	await expect(rail.locator(".cost-v")).toHaveText("$310");

	// Settings belong to the pricer: they move the figure, not the plan.
	await slider.fill("2");
	await expect(rail.locator(".cost-v")).toHaveText("$380");
	await expect(page.getByRole("button", { name: "Use guess (1)" })).toBeVisible();
	await rail.getByRole("radiogroup", { name: "Clone" }).getByRole("radio", { name: "Off" }).click();
	await expect(rail.locator(".cost-v")).toHaveText("$295");

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
