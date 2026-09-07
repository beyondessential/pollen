// Which required actions a reader has ticked off, kept per artifact in this
// browser.
//
// Progress belongs to the person working through the list, not to the artifact.
// A finalised artifact is immutable and its link is held by several people at
// once, so a tick is never stored against it and never leaves this device. A
// missing or unreadable store simply means nothing is ticked.

const KEY = "pollen.doneItems";
const MAX = 20;

type DoneRecord = {
	/** Artifact id the ticks belong to. */
	plan: string;
	/** Ids of the consequences ticked off. */
	items: string[];
	/** Epoch millis of the last change; the list is ordered by this. */
	savedAt: number;
};

export function listDone(plan: string): string[] {
	return readAll().find((r) => r.plan === plan)?.items ?? [];
}

/** Replace the ticks for one artifact, moving it to the front of the list. */
export function recordDone(plan: string, items: string[]): void {
	try {
		const next = [
			{ plan, items, savedAt: Date.now() },
			...readAll().filter((r) => r.plan !== plan),
		].slice(0, MAX);
		localStorage.setItem(KEY, JSON.stringify(next));
	} catch {
		// localStorage unavailable or full; ticking is a convenience, so skip it.
	}
}

function readAll(): DoneRecord[] {
	try {
		const raw = localStorage.getItem(KEY);
		const parsed: unknown = raw ? JSON.parse(raw) : [];
		return Array.isArray(parsed) ? parsed.filter(isDoneRecord) : [];
	} catch {
		return [];
	}
}

function isDoneRecord(r: unknown): r is DoneRecord {
	if (typeof r !== "object" || r === null) return false;
	const d = r as Record<string, unknown>;
	return (
		typeof d.plan === "string" &&
		Array.isArray(d.items) &&
		d.items.every((i) => typeof i === "string")
	);
}
