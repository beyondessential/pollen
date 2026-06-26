import { useEffect, useRef, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";

import { callApi } from "../api";

/// Landing route: creates a draft (optionally previewing a `?config` branch),
/// then collapses the URL to the draft's own id.
export default function NewApplication() {
	const [params] = useSearchParams();
	const navigate = useNavigate();
	const started = useRef(false);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		if (started.current) return; // guard StrictMode's double-invoke
		started.current = true;
		const configBranch = params.get("config") ?? undefined;
		callApi("applications", "create", { config_branch: configBranch })
			.then((view) => navigate(`/a/${view.id}`, { replace: true }))
			.catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)));
	}, [params, navigate]);

	return (
		<div className="splash">
			{error ? `Couldn't start a plan: ${error}` : "Starting a new deployment plan…"}
		</div>
	);
}
