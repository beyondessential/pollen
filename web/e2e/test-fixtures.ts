// Shared Playwright test object: every spec gets a worker-scoped pollen-server
// + Vite + per-worker database, with baseURL wired to the Vite URL.

import { test as base, expect } from "@playwright/test";

import { type StackHandle, startStack } from "./fixture";

export const test = base.extend<Record<string, never>, { stack: StackHandle }>({
	stack: [
		async ({}, use) => {
			const handle = await startStack();
			try {
				await use(handle);
			} finally {
				await handle.stop();
			}
		},
		{ scope: "worker" },
	],
	baseURL: async ({ stack }, use) => {
		await use(stack.baseUrl);
	},
});

export { expect };
