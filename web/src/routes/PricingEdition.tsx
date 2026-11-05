import { useEffect, useState } from "react";
import { Route, Routes, useParams } from "react-router-dom";

import { NotFound, Shell } from "../components/Shell";
import { type Edition, EditionProvider, callPricing } from "../edition";
import type { PricingModelView } from "../types";
import ApplicationPage from "./ApplicationPage";
import Calculator from "./Calculator";
import NewApplication from "./NewApplication";

/// The internal pricing edition (spec COST). The path segment is checked by
/// the server: until the model loads under it, nothing renders, and a path the
/// server does not recognise shows the ordinary not-found page.
export default function PricingEdition() {
	const { pricingPath = "" } = useParams();
	const [model, setModel] = useState<PricingModelView | null>(null);
	const [failed, setFailed] = useState(false);

	useEffect(() => {
		const controller = new AbortController();
		setModel(null);
		setFailed(false);
		callPricing(pricingPath, "model", {}, controller.signal)
			.then(setModel)
			.catch(() => {
				if (!controller.signal.aborted) setFailed(true);
			});
		return () => controller.abort();
	}, [pricingPath]);

	// Nothing in the internal edition's address should travel with a link
	// followed from it.
	useEffect(() => {
		const meta = document.createElement("meta");
		meta.name = "referrer";
		meta.content = "no-referrer";
		document.head.appendChild(meta);
		return () => meta.remove();
	}, []);

	if (failed) return <NotFound />;
	if (!model) return <div className="splash">Loading…</div>;

	const edition: Edition = { base: `/${pricingPath}`, pricingPath, model };
	return (
		<EditionProvider value={edition}>
			<Shell>
				<Routes>
					<Route index element={<Calculator model={model} />} />
					<Route path="new" element={<NewApplication />} />
					<Route path="a/:id" element={<ApplicationPage />} />
					<Route path="*" element={<div className="splash">Not found.</div>} />
				</Routes>
			</Shell>
		</EditionProvider>
	);
}
