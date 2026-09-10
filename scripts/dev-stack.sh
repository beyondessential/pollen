#!/usr/bin/env bash
# Local dev stack for manual testing: migrate, then run the API + Vite against
# whatever DATABASE_URL is in the environment (e.g. the ramdisk cluster). Kept
# out of the way of `just watch-*`; this is a one-shot "run it so I can click".
set -euo pipefail

# PORT is set in some shells (e.g. the workhorse local server) and clap treats
# it as present, which conflicts with the API's BIND_ADDRESS. Drop it here.
unset PORT

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export PUBLIC_BASE_URL="http://localhost:8090"

echo "dev-stack: running migrations" >&2
target/debug/migrate

echo "dev-stack: starting API on 127.0.0.1:8080" >&2
BIND_ADDRESS=127.0.0.1:8080 target/debug/pollen-server &
API_PID=$!

echo "dev-stack: starting Vite on http://localhost:8090" >&2
( cd web && npm run dev -- --host 127.0.0.1 ) &
WEB_PID=$!

trap 'kill "$API_PID" "$WEB_PID" 2>/dev/null || true' EXIT INT TERM
wait
