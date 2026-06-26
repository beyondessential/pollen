// End-to-end fixture: spawns a fresh pollen-server + Vite dev server pair
// pointed at a freshly-migrated, per-run Postgres database, and returns a
// handle Playwright tests drive.
//
// Assumes target/debug/{pollen-server,migrate} are built (it does not run
// cargo) and a Postgres reachable via POLLEN_E2E_ADMIN_DATABASE_URL (the
// `just test-e2e` wrapper points this at the ramdisk cluster). Set
// POLLEN_E2E_VERBOSE=1 to stream server/Vite output.

import { type ChildProcessWithoutNullStreams, spawn, spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import { existsSync } from "node:fs";
import net from "node:net";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const FRONTEND_ROOT = resolve(__dirname, "..");
const WORKSPACE_ROOT = resolve(FRONTEND_ROOT, "..");

export interface StackHandle {
	/** Vite dev server URL (serves the SPA, proxies /api to the binary). */
	baseUrl: string;
	/** pollen-server API URL. */
	apiUrl: string;
	stop: () => Promise<void>;
}

function getFreePort(): Promise<number> {
	return new Promise((res, rej) => {
		const srv = net.createServer();
		srv.unref();
		srv.on("error", rej);
		srv.listen(0, "127.0.0.1", () => {
			const addr = srv.address();
			if (addr && typeof addr === "object") {
				const { port } = addr;
				srv.close(() => res(port));
			} else {
				srv.close();
				rej(new Error("failed to allocate port"));
			}
		});
	});
}

async function waitForHttp(url: string, timeoutMs = 60_000): Promise<void> {
	const deadline = Date.now() + timeoutMs;
	let lastErr: unknown;
	while (Date.now() < deadline) {
		try {
			const res = await fetch(url, { redirect: "manual" });
			if (res.status > 0) return;
		} catch (e) {
			lastErr = e;
		}
		await new Promise((r) => setTimeout(r, 100));
	}
	throw new Error(`timed out waiting for ${url}: ${String(lastErr)}`);
}

function pipeOutput(child: ChildProcessWithoutNullStreams, label: string): void {
	const verbose = process.env.POLLEN_E2E_VERBOSE === "1";
	const onChunk = (chunk: Buffer, sink: NodeJS.WriteStream) => {
		if (verbose) sink.write(`[${label}] ${chunk.toString()}`);
	};
	child.stdout.on("data", (c: Buffer) => onChunk(c, process.stdout));
	child.stderr.on("data", (c: Buffer) => onChunk(c, process.stderr));
}

async function killGracefully(proc: ChildProcessWithoutNullStreams): Promise<void> {
	if (proc.exitCode !== null) return;
	proc.kill("SIGTERM");
	await new Promise<void>((res) => {
		const timer = setTimeout(() => {
			if (proc.exitCode === null) proc.kill("SIGKILL");
		}, 3000);
		proc.once("exit", () => {
			clearTimeout(timer);
			res();
		});
	});
}

function deriveDbUrl(template: string, dbName: string): string {
	const url = new URL(template);
	url.pathname = `/${dbName}`;
	return url.toString();
}

function locateBinaries() {
	const serverBin = join(WORKSPACE_ROOT, "target", "debug", "pollen-server");
	const migrateBin = join(WORKSPACE_ROOT, "target", "debug", "migrate");
	for (const bin of [serverBin, migrateBin]) {
		if (!existsSync(bin)) {
			throw new Error(`missing ${bin} — run 'cargo build --bin pollen-server --bin migrate' first`);
		}
	}
	return { serverBin, migrateBin };
}

export async function startStack(): Promise<StackHandle> {
	const { serverBin, migrateBin } = locateBinaries();

	const adminUrl = process.env.POLLEN_E2E_ADMIN_DATABASE_URL ?? "postgres://localhost/postgres";
	const dbName = `pollen_e2e_${randomBytes(6).toString("hex")}`;
	const databaseUrl = deriveDbUrl(adminUrl, dbName);

	const create = spawnSync(
		"psql",
		[adminUrl, "-v", "ON_ERROR_STOP=1", "-c", `CREATE DATABASE "${dbName}"`],
		{ encoding: "utf8" },
	);
	if (create.status !== 0) {
		throw new Error(`failed to create ${dbName} via ${adminUrl}: ${create.stderr || create.stdout}`);
	}
	const dropDb = () => {
		spawnSync(
			"psql",
			[adminUrl, "-v", "ON_ERROR_STOP=1", "-c", `DROP DATABASE IF EXISTS "${dbName}" WITH (FORCE)`],
			{ encoding: "utf8" },
		);
	};

	try {
		const migrate = spawnSync(migrateBin, [], {
			env: { ...process.env, DATABASE_URL: databaseUrl },
			encoding: "utf8",
		});
		if (migrate.status !== 0) {
			throw new Error(`migrate failed: ${migrate.stderr || migrate.stdout}`);
		}

		const apiPort = await getFreePort();
		const webPort = await getFreePort();
		const apiUrl = `http://127.0.0.1:${apiPort}`;
		const baseUrl = `http://127.0.0.1:${webPort}`;

		const api = spawn(serverBin, [], {
			env: {
				...process.env,
				DATABASE_URL: databaseUrl,
				BIND_ADDRESS: `127.0.0.1:${apiPort}`,
				POLLEN_LOG: process.env.POLLEN_LOG ?? "pollen_server=info,warn",
			},
		});
		pipeOutput(api, "api");

		const web = spawn(
			"npm",
			["run", "--silent", "dev", "--", "--port", String(webPort), "--strictPort", "--host", "127.0.0.1"],
			{ cwd: FRONTEND_ROOT, env: { ...process.env, VITE_PROXY_TARGET: apiUrl } },
		);
		pipeOutput(web, "web");

		const stop = async () => {
			await Promise.allSettled([killGracefully(web), killGracefully(api)]);
			dropDb();
		};

		try {
			await waitForHttp(`${apiUrl}/livez`);
			await waitForHttp(`${baseUrl}/`);
		} catch (e) {
			await stop();
			throw e;
		}

		return { baseUrl, apiUrl, stop };
	} catch (e) {
		dropDb();
		throw e;
	}
}
