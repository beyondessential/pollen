import { createTheme } from "@mui/material/styles";

// BES brand blues: navy carries structure, sky carries interaction.
const NAVY = "#313f6a";
const SKY = "#009cea";

export const theme = createTheme({
	palette: {
		mode: "light",
		primary: { main: NAVY },
		secondary: { main: SKY },
		background: { default: "#f4f6f8", paper: "#ffffff" },
		text: { primary: "#14222b", secondary: "#485a63" },
	},
	typography: {
		fontFamily: '"Inter", system-ui, sans-serif',
		h1: { fontFamily: '"Space Grotesk", system-ui, sans-serif' },
		h2: { fontFamily: '"Space Grotesk", system-ui, sans-serif' },
		h3: { fontFamily: '"Space Grotesk", system-ui, sans-serif' },
	},
});
