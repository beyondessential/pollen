import { Route, Routes } from "react-router-dom";

import ApplicationPage from "./routes/ApplicationPage";
import NewApplication from "./routes/NewApplication";

export default function App() {
	return (
		<>
			<header className="topbar">
				<div className="brand">
					<span className="brand-mark">BES</span>
					<span className="brand-rule" />
					<span className="brand-name">New Tamanu</span>
				</div>
			</header>
			<Routes>
				<Route path="/" element={<NewApplication />} />
				<Route path="/a/:id" element={<ApplicationPage />} />
				<Route path="*" element={<div className="splash">Not found.</div>} />
			</Routes>
		</>
	);
}
