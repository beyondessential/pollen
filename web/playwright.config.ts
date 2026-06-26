import { defineConfig, devices } from "@playwright/test";

// The per-worker fixture (e2e/fixture.ts) spawns its own pollen-server + Vite
// pair against a freshly-migrated Postgres database, so there's no global
// webServer or static baseURL here — tests get baseURL from the `stack` fixture.
export default defineConfig({
	testDir: "./e2e",
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI ? 1 : undefined,
	reporter: [["list"]],
	timeout: 60_000,
	expect: { timeout: 10_000 },
	use: {
		trace: "retain-on-failure",
		screenshot: "only-on-failure",
	},
	projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
