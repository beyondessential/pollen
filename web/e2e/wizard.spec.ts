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

const CATCHMENT = "How many people does the health system serve?";
const FACILITIES = "How many facilities will use Tamanu?";
const MOBILE = "How many people will use Tamanu on a phone or tablet?";
const DASHBOARDS = "Do you want dashboards and reporting?";
const INTEGRATIONS = "Does Tamanu need to connect to other systems?";

// Everything a plan needs to leave nothing open. The rest of the flow either
// assumes a blessed-path answer or is reached through the technical section,
// which this deliberately never opens.
const COMPLETE: Array<[string, string]> = [
	[CATCHMENT, "Under 1,000"],
	[FACILITIES, "One"],
	[MOBILE, "None"],
	[DASHBOARDS, "No, Tamanu only"],
	[INTEGRATIONS, "No integrations"],
];

async function finaliseCompletePlan(page: Page) {
	await page.goto("/");
	await expect(page).toHaveURL(/\/a\//); // URL collapses to the new draft's id
	for (const [question, option] of COMPLETE) {
		await answer(page, question, option);
	}
	await page.getByRole("button", { name: "Finalise", exact: true }).click();
	await expect(page.getByRole("heading", { name: "Tiny deployment" })).toBeVisible();
}

test("the essentials alone are enough to finalise an interim plan", async ({ page }) => {
	await page.goto("/");
	await expect(page).toHaveURL(/\/a\//);
	// The flow opens on sizing, not on anything technical.
	await expect(page.getByRole("heading", { name: CATCHMENT })).toBeVisible();
	await expect(page.getByRole("button", { name: "Finalise" })).toBeDisabled();

	// These two are the whole of what must be answered.
	await answer(page, CATCHMENT, "Under 1,000");
	await answer(page, FACILITIES, "One");

	const finalise = page.getByRole("button", { name: "Finalise interim plan" });
	await expect(finalise).toBeEnabled();
	await finalise.click();

	// The questions left open are carried on the artifact rather than guessed.
	await expect(page.getByRole("heading", { name: "Tiny deployment" })).toBeVisible();
	await expect(page.locator(".badge-interim")).toBeVisible();
	await expect(page.getByRole("heading", { name: "To confirm with BES" })).toBeVisible();
	await expect(page.getByRole("heading", { name: "Assumptions" })).toBeVisible();
});

test("walks a complete plan to a finalised artifact", async ({ page }) => {
	await page.goto("/");
	await expect(page).toHaveURL(/\/a\//);

	for (const [question, option] of COMPLETE) {
		await answer(page, question, option);
	}

	// Nothing off the standard path, so the rail stays quiet and the plan
	// finalises without being interim.
	await expect(page.getByText("Off the standard path")).toBeHidden();
	const finalise = page.getByRole("button", { name: "Finalise", exact: true });
	await expect(finalise).toBeEnabled();
	await finalise.click();

	await expect(page.getByRole("heading", { name: "Tiny deployment" })).toBeVisible();
	await expect(page.locator(".badge-interim")).toBeHidden();
	// Grouped by the reader each item is addressed to, with no warnings to show.
	await expect(page.getByRole("button", { name: "Client IT: required actions" })).toBeVisible();
	await expect(
		page.getByRole("button", { name: "BES pricing and partnerships: cost and SLA" }),
	).toBeVisible();
	await expect(page.getByRole("button", { name: "Warnings" })).toBeHidden();
	// The standard domain arrangement is BES's to set up.
	await expect(page.getByText("Provision the tamanu.app name and its certificates")).toBeVisible();

	// Compute requirements list the classes this deployment needs provisioned:
	// client-hosted facilities and the workstations staff use. Central is BES
	// cloud here, and there are no mobile users, so neither appears.
	const compute = page.locator("#s-compute");
	await expect(compute.getByRole("heading", { name: "Compute requirements" })).toBeVisible();
	await expect(compute.getByText("Facility server", { exact: true })).toBeVisible();
	await expect(compute.getByText("User devices", { exact: true })).toBeVisible();
	await expect(compute.getByText("Central server", { exact: true })).toBeHidden();
	await expect(compute.getByText("Mobile devices", { exact: true })).toBeHidden();
});

test("'Make changes' opens the new version in a new tab", async ({ page, context }) => {
	await finaliseCompletePlan(page);

	const popupPromise = context.waitForEvent("page");
	await page.getByRole("button", { name: "Make changes" }).click();
	const popup = await popupPromise;

	// The new tab holds a fresh editable draft, distinct from the artifact tab.
	await expect(popup).toHaveURL(/\/a\//);
	await expect(popup.getByRole("button", { name: "Finalise" })).toBeVisible();
	expect(popup.url()).not.toBe(page.url());
	// The original tab still shows the finalised artifact.
	await expect(page.getByRole("heading", { name: "Tiny deployment" })).toBeVisible();
});

test("a fresh plan offers to resume the previous one", async ({ page }) => {
	// First plan: a decision so it's remembered in local storage.
	await page.goto("/");
	await expect(page).toHaveURL(/\/a\//);
	await answer(page, DASHBOARDS, "No, Tamanu only");

	// A fresh plan offers to resume it; making a decision records the new plan
	// and dismisses the offer.
	await page.getByRole("link", { name: "Start a new plan" }).click();
	await expect(page).toHaveURL(/\/a\//);
	await expect(page.getByRole("button", { name: "Resume" })).toBeVisible();
	await answer(page, DASHBOARDS, "Yes, include dashboards and reporting");
	const second = page.url();
	await expect(page.getByRole("button", { name: "Resume" })).toBeHidden();

	// Resuming from another fresh plan returns to the most recent (the second),
	// with its saved answer shown selected (not a blank form).
	await page.getByRole("link", { name: "Start a new plan" }).click();
	await page.getByRole("button", { name: "Resume" }).click();
	await expect(page).toHaveURL(second);
	await expect(
		page.locator(".choice.on").filter({ hasText: "Yes, include dashboards and reporting" }),
	).toBeVisible();
});

test("a blocking conflict shows, and the sizing questions still gate finalising", async ({
	page,
}) => {
	await page.goto("/");
	await answer(page, DASHBOARDS, "Yes, include dashboards and reporting");

	// Backups sit in the technical section, which arrives collapsed.
	await page.getByRole("button", { name: "Answer more for a more accurate plan" }).click();
	await answer(page, "Can BES take backups of the data?", "No, BES may not take backups");

	// The conflict is raised, but the sizing questions are still unanswered.
	await expect(page.getByText("Backups disabled, but dashboards requested")).toBeVisible();
	await expect(page.getByRole("button", { name: "Finalise" })).toBeDisabled();
	await expect(page.getByText(/Still needed/)).toBeVisible();
});
