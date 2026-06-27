import { type ReactNode, useEffect, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";

import { callApi, useApi } from "../api";
import Artifact from "../components/Artifact";
import Wizard from "../components/Wizard";
import { type RecentPlan, listRecentPlans, recordRecentPlan } from "../recentPlans";
import type { AnswerValue, AppView } from "../types";

export default function ApplicationPage() {
	const { id = "" } = useParams();
	const state = useApi("applications", "get", { id }, [id]);

	if (state.status === "error") {
		return <div className="splash">Couldn't load this plan: {state.error.message}</div>;
	}
	if (state.status !== "ok") {
		return <div className="splash">Loading…</div>;
	}
	// Re-key on id so local edit state resets when navigating between artifacts.
	return <Loaded key={id} initial={state.data} />;
}

function Loaded({ initial }: { initial: AppView }) {
	const [view, setView] = useState(initial);
	const [params] = useSearchParams();
	const navigate = useNavigate();
	const [busy, setBusy] = useState(false);
	const [resumeDismissed, setResumeDismissed] = useState(false);

	const answers = view.answers as unknown as Record<string, AnswerValue>;
	const started = Object.values(answers).some((v) =>
		Array.isArray(v) ? v.length > 0 : v != null && v !== "",
	);
	const size = view.evaluation.derived["size"] ?? null;

	// The most recent *other* plan, captured once on mount — before we record
	// this one below — so a freshly-started plan can offer to resume it.
	const [previous] = useState<RecentPlan | undefined>(() =>
		listRecentPlans().find((p) => p.id !== initial.id),
	);

	// Remember a plan once it carries a decision (or is finalised), so it's the
	// one we'd resume next time. A brand-new, untouched draft is deliberately not
	// recorded — otherwise it would propose resuming itself.
	useEffect(() => {
		if (view.status === "finalised" || started) {
			recordRecentPlan({ id: view.id, status: view.status, size });
		}
	}, [view.id, view.status, started, size]);

	// A `?config=<branch>` on an existing plan's URL offers to switch it to that
	// previewed ruleset. Otherwise a draft bound to a stale default offers to
	// update. Either is a fork (the parent is left untouched) that migrates the
	// answers, then we land on the new version.
	const configBranch = params.get("config") ?? undefined;

	async function rebind(args: { config_branch?: string; to_default?: boolean }) {
		setBusy(true);
		try {
			const forked = await callApi("applications", "fork", { id: view.id, ...args });
			navigate(`/a/${forked.id}`, { replace: true });
		} finally {
			setBusy(false);
		}
	}

	let banner: ReactNode = null;
	if (configBranch) {
		banner = (
			<UpdateBar
				busy={busy}
				action="Switch this plan to it"
				onAct={() => rebind({ config_branch: configBranch })}
			>
				Viewing against the <code>{configBranch}</code> ruleset preview.
			</UpdateBar>
		);
	} else if (view.update_available) {
		banner = (
			<UpdateBar
				busy={busy}
				action="Update to the latest"
				onAct={() => rebind({ to_default: true })}
			>
				The ruleset has been updated since this plan started.
			</UpdateBar>
		);
	}

	// Offer to resume only on a fresh, untouched draft, and not while a
	// config/update banner is already showing. Making a decision flips
	// `started`, which records this plan and hides the offer.
	const showResume =
		!banner && !resumeDismissed && !started && view.status === "draft" && previous != null;

	return (
		<>
			{banner}
			{showResume && previous != null && (
				<ResumeBar
					plan={previous}
					onResume={() => navigate(`/a/${previous.id}`)}
					onDismiss={() => setResumeDismissed(true)}
				/>
			)}
			{view.status === "finalised" ? (
				<Artifact view={view} />
			) : (
				<Wizard view={view} setView={setView} />
			)}
		</>
	);
}

function ResumeBar({
	plan,
	onResume,
	onDismiss,
}: {
	plan: RecentPlan;
	onResume: () => void;
	onDismiss: () => void;
}) {
	const what = plan.status === "finalised" ? "finalised" : "in progress";
	const detail = plan.size ? `${plan.size} · ${what}` : what;
	return (
		<div className="updatebar">
			<span>
				Pick up where you left off? <span className="updatebar-sub">{detail}</span>
			</span>
			<div className="updatebar-actions">
				<button type="button" className="btn primary sm" onClick={onResume}>
					Resume
				</button>
				<button type="button" className="btn ghost sm" onClick={onDismiss}>
					Dismiss
				</button>
			</div>
		</div>
	);
}

function UpdateBar({
	children,
	action,
	busy,
	onAct,
}: {
	children: ReactNode;
	action: string;
	busy: boolean;
	onAct: () => void;
}) {
	return (
		<div className="updatebar">
			<span>{children}</span>
			<button type="button" className="btn primary sm" disabled={busy} onClick={onAct}>
				{action}
			</button>
		</div>
	);
}
