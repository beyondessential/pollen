import { useState } from "react";
import { useParams } from "react-router-dom";

import { useApi } from "../api";
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
	return view.status === "finalised" ? (
		<Artifact view={view} />
	) : (
		<Wizard view={view} setView={setView} />
	);
}
