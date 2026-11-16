import { useEffect, useState } from "react";

import { NotFound, Shell } from "../components/Shell";
import { type Edition, EditionProvider, callMigration } from "../edition";
import type { MigrationModelView } from "../types";
import MigrationEstimator from "./MigrationEstimator";

/// The internal data migration estimator (spec MIG). The path segment is
/// checked by the server: until the model loads under it, nothing renders, and
/// a path the server does not recognise shows the ordinary not-found page.
export default function MigrationEdition({ path }: { path: string }) {
	const [model, setModel] = useState<MigrationModelView | null>(null);
	const [failed, setFailed] = useState(false);

	useEffect(() => {
		const controller = new AbortController();
		setModel(null);
		setFailed(false);
		callMigration(path, "model", {}, controller.signal)
			.then(setModel)
			.catch(() => {
				if (!controller.signal.aborted) setFailed(true);
			});
		return () => controller.abort();
	}, [path]);

	if (failed) return <NotFound />;
	if (!model) return <div className="splash">Loading…</div>;

	const edition: Edition = { base: `/${path}`, pricingPath: null, model: null, migrationPath: path };
	return (
		<EditionProvider value={edition}>
			<Shell>
				<MigrationEstimator model={model} />
			</Shell>
		</EditionProvider>
	);
}
