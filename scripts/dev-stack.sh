#!/usr/bin/env bash
# Local dev stack for manual testing: migrate, then run the API + Vite against
# whatever DATABASE_URL is in the environment (e.g. the ramdisk cluster).
#
# Ports are deliberately NOT the canonical 8080/8090: other worktrees of this
# repo often have `just watch-api`/`watch-web` running there, and a Vite on
# ::1:8090 plus one on 127.0.0.1:8090 can coexist and silently serve the wrong
# worktree. Override with API_PORT / WEB_PORT.
set -euo pipefail

# PORT is set in some shells (e.g. the workhorse local server) and clap treats
# it as present, which conflicts with the API's BIND_ADDRESS. Drop it here.
unset PORT

API_PORT="${API_PORT:-8081}"
WEB_PORT="${WEB_PORT:-8091}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export PUBLIC_BASE_URL="http://127.0.0.1:${WEB_PORT}"

echo "dev-stack: running migrations" >&2
target/debug/migrate

echo "dev-stack: starting API on 127.0.0.1:${API_PORT}" >&2
BIND_ADDRESS="127.0.0.1:${API_PORT}" target/debug/pollen-server &
API_PID=$!

echo "dev-stack: starting Vite on http://127.0.0.1:${WEB_PORT}" >&2
(
  cd web
  VITE_PROXY_TARGET="http://127.0.0.1:${API_PORT}" \
    npm run dev -- --port "$WEB_PORT" --strictPort --host 127.0.0.1
) &
WEB_PID=$!

trap 'kill "$API_PID" "$WEB_PID" 2>/dev/null || true' EXIT INT TERM
wait
