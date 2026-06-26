use std::path::Path;
use std::process::Command;
use std::{env, fs};

// Builds and embeds the web SPA (web/dist) at compile time. In dev the Vite
// server is the real source of the UI, so SKIP_FRONTEND_BUILD=1 skips this and
// the binary just embeds whatever is on disk.
fn main() {
	let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let frontend = Path::new(&manifest_dir).join("../../web");
	let dist = frontend.join("dist");

	println!("cargo:rerun-if-changed=../../web/src");
	println!("cargo:rerun-if-changed=../../web/public");
	println!("cargo:rerun-if-changed=../../web/package.json");
	println!("cargo:rerun-if-changed=../../web/package-lock.json");
	println!("cargo:rerun-if-changed=../../web/vite.config.ts");
	println!("cargo:rerun-if-changed=../../web/index.html");
	println!("cargo:rerun-if-changed=../../web/tsconfig.app.json");
	println!("cargo:rerun-if-env-changed=SKIP_FRONTEND_BUILD");

	// Ensure dist/ exists so rust-embed has something to point at even when the
	// build below is skipped.
	fs::create_dir_all(&dist).expect("failed to create web/dist");

	if env::var("SKIP_FRONTEND_BUILD").is_ok_and(|v| !v.is_empty()) {
		return;
	}

	let Some(npm) = which_npm() else {
		println!(
			"cargo:warning=npm not found; using whatever is in web/dist (set SKIP_FRONTEND_BUILD=1 to silence this)"
		);
		return;
	};

	let status = Command::new(&npm)
		.arg("ci")
		.current_dir(&frontend)
		.status()
		.expect("failed to run npm ci");
	assert!(status.success(), "npm ci failed");

	let status = Command::new(&npm)
		.args(["run", "build"])
		.current_dir(&frontend)
		.status()
		.expect("failed to run npm run build");
	assert!(status.success(), "npm run build failed");
}

fn which_npm() -> Option<String> {
	for candidate in ["npm", "npm.cmd"] {
		if Command::new(candidate)
			.arg("--version")
			.output()
			.is_ok_and(|o| o.status.success())
		{
			return Some(candidate.to_owned());
		}
	}
	None
}
