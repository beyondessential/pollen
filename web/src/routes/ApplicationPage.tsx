import { type ReactNode, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";

import { callApi, useApi } from "../api";
import Artifact from "../components/Artifact";
import Wizard from "../components/Wizard";
import type { AppView } from "../types";

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

	return (
		<>
			{banner}
			{view.status === "finalised" ? (
				<Artifact view={view} />
			) : (
				<Wizard view={view} setView={setView} />
			)}
		</>
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
