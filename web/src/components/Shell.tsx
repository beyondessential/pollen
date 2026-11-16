import type { ReactNode } from "react";
import { Link, NavLink } from "react-router-dom";

import besLogo from "../assets/bes-logo.png";
import { useEdition } from "../edition";

export function NotFound() {
	return (
		<Shell>
			<div className="splash">Not found.</div>
		</Shell>
	);
}

export function Shell({ children }: { children: ReactNode }) {
	const edition = useEdition();
	const internal = edition.pricingPath != null;
	// The data migration estimator is a single page with no plans beneath it.
	const migration = edition.migrationPath != null;
	return (
		<>
			<header className="topbar">
				<Link to={internal || migration ? edition.base : "/"} className="brand">
					<img src={besLogo} className="brand-logo" alt="BES" />
					<span className="brand-rule" />
					<span className="brand-name">New Tamanu</span>
					{internal && <span className="brand-tag">Internal pricing</span>}
					{migration && <span className="brand-tag">Internal migration estimates</span>}
				</Link>
				<nav className="topbar-nav">
					{(internal || migration) && (
						<NavLink
							to={edition.base}
							end
							className={({ isActive }) => `topbar-new${isActive ? " active" : ""}`}
						>
							{migration ? "Estimator" : "Calculator"}
						</NavLink>
					)}
					{!migration && (
						<Link to={internal ? `${edition.base}/new` : "/"} className="topbar-new">
							Start a new plan
						</Link>
					)}
				</nav>
			</header>
			{children}
		</>
	);
}
