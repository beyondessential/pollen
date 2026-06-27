// A small, bounded list of recently-touched plans, kept in localStorage so a
// freshly-started plan can offer to resume the previous one. Stored as a list
// (most-recent first) rather than a single entry, so it can be surfaced more
// fully later. Purely a client-side convenience: the server is the source of
// truth, and a missing or unreadable store just means no resume offer.

const KEY = "pollen.recentPlans";
const MAX = 12;

export type RecentPlan = {
	id: string;
	status: "draft" | "finalised";
	/** Derived size band, if known, for a recognisable label. */
	size: string | null;
	/** Epoch millis of the last touch; the list is ordered by this. */
	savedAt: number;
};

export function listRecentPlans(): RecentPlan[] {
	try {
		const raw = localStorage.getItem(KEY);
		const parsed: unknown = raw ? JSON.parse(raw) : [];
		return Array.isArray(parsed) ? parsed.filter(isRecentPlan) : [];
	} catch {
		return [];
	}
}

/** Upsert a plan to the front of the list (deduped by id), capped at MAX. */
export function recordRecentPlan(plan: Omit<RecentPlan, "savedAt">): void {
	try {
		const next = [
			{ ...plan, savedAt: Date.now() },
			...listRecentPlans().filter((p) => p.id !== plan.id),
		].slice(0, MAX);
		localStorage.setItem(KEY, JSON.stringify(next));
	} catch {
		// localStorage unavailable or full — resume is a convenience, so skip it.
	}
}

function isRecentPlan(p: unknown): p is RecentPlan {
	if (typeof p !== "object" || p === null) return false;
	const r = p as Record<string, unknown>;
	return typeof r.id === "string" && (r.status === "draft" || r.status === "finalised");
}
