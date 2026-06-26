import { createTheme } from "@mui/material/styles";

// Placeholder palette — real visual design lands with the wizard UI. Pollen:
// a warm amber against a leaf green.
const POLLEN = "#c8951b";
const POLLEN_LIGHT = "#e0b24a";
const LEAF = "#2f7d4f";
const LEAF_LIGHT = "#56a877";

export function makeTheme(mode: "light" | "dark") {
	const dark = mode === "dark";
	return createTheme({
		palette: {
			mode,
			primary: { main: dark ? POLLEN_LIGHT : POLLEN },
			secondary: { main: dark ? LEAF_LIGHT : LEAF },
		},
	});
}
