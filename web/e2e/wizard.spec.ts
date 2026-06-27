import type { Page } from "@playwright/test";

import { expect, test } from "./test-fixtures";

// Click the option labelled `option` within the question card headed by
// `question`, then wait for the answer to round-trip so the next click lands
// on a settled view.
async function answer(page: Page, question: string, option: string) {
	const card = page.locator(".card", { hasText: question });
	const patched = page.waitForResponse((r) => r.url().includes("/api/applications/patch"));
	await card.getByRole("button", { name: option }).first().click();
	await patched;
}

// A complete, all-default configuration (no off-default choices → clear verdict).
const CLEAR: Array<[string, string]> = [
	["Connect to Tupaia?", "No Tupaia"],
	["Which integrations are wanted?", "None for now"],
	["Catchment population", "Under 5,000"],
	["Number of facilities", "1 – 5"],
	["Mobile clients", "Under 10"],
	["Where does the central server run?", "BES cloud"],
	["What's the facility mix?", "Some in BES cloud"],
	["Which hosting region?", "Sydney"],
	["Can BES take backups?", "Yes — BES may take backups"],
	["What does BES retain?", "Full retention"],
	["How often do you intend to upgrade?", "Every release"],
	["DNS authority", "BES controls DNS"],
	["How is the BES-controlled domain set up?", "BES subdomain on tamanu.app"],
	["Remote access for managed servers", "Tailscale"],
	["Time synchronisation", "Public NTP"],
];

// Walk the all-default plan and finalise it, landing on the artifact.
async function finaliseDefaultPlan(page: Page) {
	await page.goto("/");
	await expect(page).toHaveURL(/\/a\//); // URL collapses to the new draft's id
	for (const [question, option] of CLEAR) {
		await answer(page, question, option);
	}
	await page.getByRole("button", { name: "Finalise" }).click();
	await expect(page.getByRole("heading", { name: "Small deployment" })).toBeVisible();
}

test("walks a default plan to a finalised artifact", async ({ page }) => {
	await page.goto("/");
	await expect(page).toHaveURL(/\/a\//); // URL collapses to the new draft's id
	await expect(page.getByRole("heading", { name: "Connect to Tupaia?" })).toBeVisible();

	for (const [question, option] of CLEAR) {
		await answer(page, question, option);
	}

	// All visible questions answered → clear verdict, finalise enabled.
	await expect(page.getByText("On the default, supported path")).toBeVisible();
	const finalise = page.getByRole("button", { name: "Finalise" });
	await expect(finalise).toBeEnabled();
	await finalise.click();

	// The finalised artifact.
	await expect(page.getByRole("heading", { name: "Small deployment" })).toBeVisible();

	// By-topic grouping and search both work.
	await page.getByRole("button", { name: "By topic" }).click();
	await expect(page.getByRole("heading", { name: "Networking" })).toBeVisible();
	await page.getByPlaceholder("Search consequences…").fill("Tailscale");
	await expect(page.getByText("Allow BES's Tailscale on managed servers")).toBeVisible();
});

test("'Make changes' opens the new version in a new tab", async ({ page, context }) => {
	await finaliseDefaultPlan(page);

	const popupPromise = context.waitForEvent("page");
	await page.getByRole("button", { name: "Make changes" }).click();
	const popup = await popupPromise;

	// The new tab holds a fresh editable draft, distinct from the artifact tab.
	await expect(popup).toHaveURL(/\/a\//);
	await expect(popup.getByRole("button", { name: "Finalise" })).toBeVisible();
	expect(popup.url()).not.toBe(page.url());
	// The original tab still shows the finalised artifact.
	await expect(page.getByRole("heading", { name: "Small deployment" })).toBeVisible();
});

test("a fresh plan offers to resume the previous one", async ({ page }) => {
	// First plan: a decision so it's remembered in local storage.
	await page.goto("/");
	await expect(page).toHaveURL(/\/a\//);
	await answer(page, "Connect to Tupaia?", "No Tupaia");

	// A fresh plan offers to resume it; making a decision records the new plan
	// and dismisses the offer.
	await page.getByRole("link", { name: "Start a new plan" }).click();
	await expect(page).toHaveURL(/\/a\//);
	await expect(page.getByRole("button", { name: "Resume" })).toBeVisible();
	await answer(page, "Connect to Tupaia?", "Yes, connect to Tupaia");
	const second = page.url();
	await expect(page.getByRole("button", { name: "Resume" })).toBeHidden();

	// Resuming from another fresh plan returns to the most recent (the second),
	// with its saved answer shown selected (not a blank form).
	await page.getByRole("link", { name: "Start a new plan" }).click();
	await page.getByRole("button", { name: "Resume" }).click();
	await expect(page).toHaveURL(second);
	await expect(
		page.locator(".choice.on").filter({ hasText: "Yes, connect to Tupaia" }),
	).toBeVisible();
});

test("a blocking, incomplete plan can't be finalised", async ({ page }) => {
	await page.goto("/");
	await answer(page, "Connect to Tupaia?", "Yes, connect to Tupaia");
	await answer(page, "Can BES take backups?", "No — BES may not take backups");

	// The conflict shows, but the form is incomplete, so finalise stays disabled.
	await expect(page.getByText("Not possible as specified")).toBeVisible();
	await expect(page.getByRole("button", { name: "Finalise" })).toBeDisabled();
});
