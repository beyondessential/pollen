import { Outlet, Route, Routes } from "react-router-dom";

import { NotFound, Shell } from "./components/Shell";
import ApplicationPage from "./routes/ApplicationPage";
import NewApplication from "./routes/NewApplication";
import PricingEdition from "./routes/PricingEdition";

export default function App() {
	return (
		<Routes>
			<Route
				element={
					<Shell>
						<Outlet />
					</Shell>
				}
			>
				<Route path="/" element={<NewApplication />} />
				<Route path="/a/:id" element={<ApplicationPage />} />
			</Route>
			{/* The internal edition, under its unguessable path. A path that is not
			    the configured one reads as not found. */}
			<Route path="/:pricingPath/*" element={<PricingEdition />} />
			<Route path="*" element={<NotFound />} />
		</Routes>
	);
}
