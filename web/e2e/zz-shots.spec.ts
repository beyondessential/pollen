import type { Page } from "@playwright/test";
import { PRICING_PATH } from "./fixture";
import { test } from "./test-fixtures";

const OUT = process.env.SHOTS_DIR ?? "/tmp/j1shots/r1";
const BASE = `/${PRICING_PATH}`;

async function answer(page: Page, q: string, o: string) {
	const p = page.waitForResponse((r) => r.url().includes("/patch"));
	await page.locator(".card", { hasText: q }).getByRole("button", { name: o }).first().click();
	await p;
}
const group = (page: Page, name: string) => page.getByRole("radiogroup", { name, exact: true });

test("calculator states", async ({ page }) => {
	await page.setViewportSize({ width: 1280, height: 900 });
	await page.goto(BASE);
	await page.waitForTimeout(400);
	await page.screenshot({ path: `${OUT}/01-calc-default.png`, fullPage: true });

	await page.getByRole("button", { name: "Estimate size" }).click();
	await page.getByRole("group", { name: "How many people does the health system serve?" }).getByRole("button", { name: "1,000 - 10,000" }).click();
	await page.getByRole("group", { name: "How many facilities will use Tamanu?" }).getByRole("button", { name: "6 - 20" }).click();
	await page.getByRole("group", { name: "Does Tamanu need to connect to other systems?" }).getByRole("button", { name: "Yes" }).click();
	await group(page, "Facility servers").getByRole("radio", { name: "A mix" }).click();
	await page.waitForTimeout(500);
	await page.screenshot({ path: `${OUT}/02-calc-estimator-mix.png`, fullPage: true });

	await page.setViewportSize({ width: 390, height: 844 });
	await page.waitForTimeout(300);
	await page.screenshot({ path: `${OUT}/03-calc-narrow.png`, fullPage: true });
});

test("wizard states", async ({ page }) => {
	await page.setViewportSize({ width: 1280, height: 900 });
	await page.goto(`${BASE}/new`);
	await page.waitForURL(/\/a\//);
	await page.waitForTimeout(400);
	await page.screenshot({ path: `${OUT}/04-wizard-unsized.png` });

	await answer(page, "How many people", "10,000 - 100,000");
	await answer(page, "How many facilities", "6 - 20");
	await answer(page, "Do you want dashboards", "I'm not sure");
	await answer(page, "Where should the facility servers run?", "A mix of both");
	await page.getByRole("heading", { name: "How many facility servers will BES host?" }).scrollIntoViewIfNeeded();
	await page.waitForTimeout(500);
	await page.screenshot({ path: `${OUT}/05-wizard-mix.png` });
	await page.evaluate(() => window.scrollTo(0, 0));
	await page.waitForTimeout(200);
	await page.screenshot({ path: `${OUT}/06-wizard-top.png` });

	await page.getByRole("button", { name: /Finalise/ }).click();
	await page.getByRole("heading", { name: /deployment/ }).first().waitFor();
	await page.waitForTimeout(500);
	await page.screenshot({ path: `${OUT}/07-wizard-finalised.png` });

	await page.setViewportSize({ width: 390, height: 844 });
	await page.goto(`${BASE}/new`);
	await page.waitForURL(/\/a\//);
	await answer(page, "How many people", "10,000 - 100,000");
	await answer(page, "How many facilities", "6 - 20");
	await page.waitForTimeout(500);
	await page.screenshot({ path: `${OUT}/08-wizard-narrow.png` });
});
