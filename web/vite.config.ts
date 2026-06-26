import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Defaults match `just watch-api`. The e2e fixture overrides this to point at
// the per-run server it spawns on a dynamic port.
const API_TARGET = process.env.VITE_PROXY_TARGET ?? "http://127.0.0.1:8080";

export default defineConfig({
	plugins: [react()],
	build: {
		outDir: "dist",
		emptyOutDir: true,
	},
	server: {
		port: 8090,
		strictPort: true,
		proxy: {
			"/api": API_TARGET,
		},
	},
});
