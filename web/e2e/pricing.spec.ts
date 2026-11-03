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
	// Tiny, BES cloud central, standard clone, backups: 242.65, shown as 245.
	await expect(figure).toHaveText("$245");
	await expect(page.locator(".calc-result .cost-annual")).toHaveText("$2,910 a year");

	await page.getByRole("radio", { name: "Medium" }).click();
	// 544.59.
	await expect(figure).toHaveText("$545");

	// Backups off removes only the backups share; the clone stays.
	await page.getByRole("radiogroup", { name: "BES takes backups" }).getByRole("radio", { name: "No" }).click();
	await expect(figure).toHaveText("$505");
	await expect(page.locator(".calc-result .cost-lines")).toContainText("Clone");
	await expect(page.locator(".calc-result .cost-lines")).not.toContainText("Backups share");

	// A client-hosted central drops the central server and its Windows toggle.
	await page.getByRole("radio", { name: "Client hosted" }).click();
	await expect(page.locator(".calc-result .cost-lines")).not.toContainText("Central server");
	await expect(
		page.getByRole("radiogroup", { name: "Windows central server" }).getByRole("radio", { name: "Yes" }),
	).toBeDisabled();

	await page.getByRole("button", { name: "Assumptions" }).click();
	await expect(page.getByText("Standard clone uptime")).toBeVisible();
	await expect(page.locator(".conf-estimated").first()).toBeVisible();
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

	// Overrides belong to the pricer: they move the figure, not the plan.
	await rail.getByRole("radiogroup", { name: "Clone" }).getByRole("radio", { name: "Off" }).click();
	await expect(rail.locator(".cost-v")).toHaveText("$160");
	await rail.getByRole("slider", { name: "Cloud facility servers" }).fill("2");
	await expect(rail.locator(".cost-v")).toHaveText("$295");
	await expect(rail.getByRole("button", { name: "Use guess (0)" })).toBeVisible();
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
