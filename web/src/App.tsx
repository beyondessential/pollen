import { Link, Route, Routes } from "react-router-dom";

import ApplicationPage from "./routes/ApplicationPage";
import NewApplication from "./routes/NewApplication";

export default function App() {
	return (
		<>
			<header className="topbar">
				<Link to="/" className="brand">
					<span className="brand-mark">BES</span>
					<span className="brand-rule" />
					<span className="brand-name">New Tamanu</span>
				</Link>
				<Link to="/" className="topbar-new">
					Start a new plan
				</Link>
			</header>
			<Routes>
				<Route path="/" element={<NewApplication />} />
				<Route path="/a/:id" element={<ApplicationPage />} />
				<Route path="*" element={<div className="splash">Not found.</div>} />
			</Routes>
		</>
	);
}
