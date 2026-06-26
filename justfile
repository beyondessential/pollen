# Pollen development commands

# Skip the npm install + build that pollen-server's build.rs runs by default.
# In dev, Vite serves the frontend directly; the binary only needs to embed
# web/dist/ for release builds. Release re-enables it (see build-release).
export SKIP_FRONTEND_BUILD := "1"

# ...for development links
export PUBLIC_BASE_URL := "http://localhost:8090"

# Show available commands
default:
    @just --list

# Check that the project compiles
check:
    cargo check

# Run the API server, bound to IPv4 so Vite's proxy can reach it, reloading on
# source change. (Node's vite-proxy can't resolve [::1] literals.)
watch-api:
    BIND_ADDRESS=127.0.0.1:8080 watchexec -I -w crates -- cargo run --bin pollen-server

# Run the Vite frontend dev server (proxies /api to watch-api)
watch-web:
    cd web && npm run dev

# Regenerate the OpenAPI spec → TypeScript types. Run after changing any
# handler's request/response shape, security scheme, or tag; commit both files.
gen-openapi:
    cargo run --quiet --bin openapi-dump > web/openapi.json
    cd web && npm run gen:api-types

# Typecheck the frontend. `-b` is load-bearing: the root tsconfig is
# references-only, so a bare `tsc --noEmit` checks nothing.
typecheck:
    cd web && npx tsc -b

# Run all tests against a throwaway RAM-backed Postgres (see scripts/ramdisk-pg.sh).
# Args pass straight to nextest.
test *args:
    scripts/ramdisk-pg.sh cargo nextest run --no-fail-fast {{ args }}

# Run any command against the throwaway RAM-backed Postgres.
fast +cmd:
    scripts/ramdisk-pg.sh {{ cmd }}

# Format / lint
fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy --all-targets

# Release build with the embedded frontend (re-enables build.rs frontend build)
build-release target:
    SKIP_FRONTEND_BUILD= cargo build --locked --target {{ target }} --release --bins

# Build the container image
build-image:
    docker build -f .github/Containerfile -t pollen .
