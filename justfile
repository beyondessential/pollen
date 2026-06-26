# Pollen development commands

# Skip the npm install + build that pollen-server's build.rs runs by default.
# In dev, Vite serves the frontend directly; the binary only needs to embed
# web/dist/ for release builds. Release re-enables it (see build-release).
export SKIP_FRONTEND_BUILD := "1"

# Local dev database (its own DB; nothing fleet-related). Override via the env.
export DATABASE_URL := env('DATABASE_URL', 'postgres://localhost/pollen')

# ...for development links
export PUBLIC_BASE_URL := "http://localhost:8090"

# Show available commands
default:
    @just --list

# Check that the project compiles
check:
    cargo check

# Run the API server, bound to IPv4 so Vite's proxy can reach it, reloading on
# source change. (Node's vite-proxy can't resolve [::1] literals.) Watches the
# repo dir and filters to crates/ + ruleset.ron (which is embedded via
# include_str!) — watching the dir, not the file, survives editor rename-saves.
watch-api:
    BIND_ADDRESS=127.0.0.1:8080 watchexec -I -w crates -W . -f 'crates/**' -f 'ruleset.ron' -- cargo run --bin pollen-server

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

# Run pending migrations against DATABASE_URL, then regenerate schema.rs.
migrate:
    diesel migration run
    cargo fmt

# Create a new migration (writes up.sql/down.sql; never hand-create these)
migration name:
    diesel migration generate {{ name }}

# Revert the last migration, then regenerate schema.rs
migrate-revert:
    diesel migration revert
    cargo fmt

# Redo the last migration (down then up)
migrate-redo:
    diesel migration redo
    cargo fmt

# Run all tests against a throwaway RAM-backed Postgres (see scripts/ramdisk-pg.sh).
# Args pass straight to nextest.
test *args:
    scripts/ramdisk-pg.sh cargo nextest run --no-fail-fast {{ args }}

# Run any command against the throwaway RAM-backed Postgres.
fast +cmd:
    scripts/ramdisk-pg.sh {{ cmd }}

# Run the Playwright e2e suite. Builds the binaries the fixture spawns, then runs
# against the throwaway RAM-backed Postgres (the wrapper sets
# POLLEN_E2E_ADMIN_DATABASE_URL). First run on a fresh checkout needs:
#   cd web && npx playwright install chromium
test-e2e:
    cargo build --bin pollen-server --bin migrate
    cd web && {{ justfile_directory() }}/scripts/ramdisk-pg.sh npm run test:e2e

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

# Build the container image locally. Prefers podman, falls back to docker.
# Rootless podman needs a subuid/subgid range to map image file ownership (e.g.
# gid 42 in Debian bases); where that isn't configured, this uses rootful podman
# if sudo is passwordless. Override explicitly with CONTAINER_ENGINE="sudo podman".
build-image:
    #!/usr/bin/env bash
    set -euo pipefail
    engine="${CONTAINER_ENGINE:-}"
    if [ -z "$engine" ]; then
        if command -v podman >/dev/null; then
            if grep -q "^$(id -un):\|^$(id -u):" /etc/subgid 2>/dev/null; then
                engine="podman"          # rootless: subgid range is configured
            elif sudo -n true 2>/dev/null; then
                engine="sudo podman"     # no subgid range; use rootful via passwordless sudo
            else
                engine="podman"          # let podman report the rootless mapping error
            fi
        elif command -v docker >/dev/null; then
            engine="docker"
        fi
    fi
    [ -n "$engine" ] || { echo "error: need podman or docker on PATH" >&2; exit 69; }
    case "$(uname -m)" in aarch64|arm64) arch=arm64;; x86_64|amd64) arch=amd64;; *) arch="$(uname -m)";; esac
    # Stage a build context mirroring CI's layout: the binary under <arch>/.
    ctx="$(mktemp -d)"; trap 'rm -rf "$ctx"' EXIT
    SKIP_FRONTEND_BUILD= cargo build --release --bin pollen-server --bin migrate
    mkdir -p "$ctx/$arch"; cp target/release/pollen-server target/release/migrate "$ctx/$arch/"
    echo "building image 'pollen' with: $engine (arch $arch)" >&2
    $engine build -f "$PWD/.github/Containerfile" --build-arg "BIN_DIR=$arch" -t pollen "$ctx"
