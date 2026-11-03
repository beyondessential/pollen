import type { ReactNode } from "react";
import { Link } from "react-router-dom";

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
	return (
		<>
			<header className="topbar">
				<Link to={internal ? edition.base : "/"} className="brand">
					<img src={besLogo} className="brand-logo" alt="BES" />
					<span className="brand-rule" />
					<span className="brand-name">New Tamanu</span>
					{internal && <span className="brand-tag">Internal pricing</span>}
				</Link>
				<nav className="topbar-nav">
					{internal && (
						<Link to={edition.base} className="topbar-new">
							Calculator
						</Link>
					)}
					<Link to={internal ? `${edition.base}/new` : "/"} className="topbar-new">
						Start a new plan
					</Link>
				</nav>
			</header>
			{children}
		</>
	);
}
