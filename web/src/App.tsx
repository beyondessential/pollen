import { Link, Route, Routes } from "react-router-dom";

import besLogo from "./assets/bes-logo.png";
import ApplicationPage from "./routes/ApplicationPage";
import NewApplication from "./routes/NewApplication";

export default function App() {
	return (
		<>
			<header className="topbar">
				<Link to="/" className="brand">
					<img src={besLogo} className="brand-logo" alt="BES" />
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
