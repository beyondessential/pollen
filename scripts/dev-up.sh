#!/usr/bin/env bash
# Bring up a DETACHED local dev stack for manual testing, and return.
#
# Unlike `just watch-*` (foreground) and scripts/dev-stack.sh (dies with its
# parent shell), everything here is orphaned to init via nohup so the stack
# survives the shell -- and the agent session -- that started it.
#
# Idempotent: re-run to repair a partially-dead stack. Stop with dev-down.sh.
#
# Ports avoid the canonical 8080/8090, which other worktrees of this repo often
# hold; a Vite on ::1:8090 and one on 127.0.0.1:8090 can coexist and silently
# serve the wrong worktree. Override with API_PORT / WEB_PORT / PG_PORT.
set -euo pipefail

unset PORT   # clap treats env PORT as present, conflicting with BIND_ADDRESS

API_PORT="${API_PORT:-8081}"
WEB_PORT="${WEB_PORT:-8091}"
PG_PORT="${PG_PORT:-5440}"
ROLE=pollen

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
RUN="$ROOT/.dev-stack"          # logs + pids (gitignored via .git/info/exclude)
PGDIR="$HOME/.cache/pollen-dev-pg"
mkdir -p "$RUN"

export DATABASE_URL="postgresql://${ROLE}@127.0.0.1:${PG_PORT}/${ROLE}"
export PUBLIC_BASE_URL="http://127.0.0.1:${WEB_PORT}"

# ── Postgres: persistent cluster, trust auth, daemonised by pg_ctl ──────────
if [ ! -s "$PGDIR/PG_VERSION" ]; then
  echo "dev-up: initialising cluster at $PGDIR" >&2
  rm -rf "$PGDIR"; mkdir -p "$PGDIR"
  initdb -D "$PGDIR" -U "$ROLE" --auth=trust -E UTF8 >/dev/null
fi
if ! pg_ctl -D "$PGDIR" status >/dev/null 2>&1; then
  echo "dev-up: starting postgres on 127.0.0.1:$PG_PORT" >&2
  pg_ctl -D "$PGDIR" -l "$PGDIR/server.log" -w start \
    -o "-p $PG_PORT -h 127.0.0.1 -k $PGDIR" >/dev/null
fi
createdb -h 127.0.0.1 -p "$PG_PORT" -U "$ROLE" "$ROLE" 2>/dev/null || true

echo "dev-up: running migrations" >&2
target/debug/migrate

# ── API + Vite, orphaned so they outlive this shell ────────────────────────
alive() { [ -f "$1" ] && kill -0 "$(cat "$1")" 2>/dev/null; }

if alive "$RUN/api.pid"; then
  echo "dev-up: API already running (pid $(cat "$RUN/api.pid"))" >&2
else
  echo "dev-up: starting API on 127.0.0.1:$API_PORT" >&2
  ( BIND_ADDRESS="127.0.0.1:${API_PORT}" \
    nohup target/debug/pollen-server >"$RUN/api.log" 2>&1 &
    echo $! > "$RUN/api.pid" )
fi

if alive "$RUN/web.pid"; then
  echo "dev-up: Vite already running (pid $(cat "$RUN/web.pid"))" >&2
else
  echo "dev-up: starting Vite on 127.0.0.1:$WEB_PORT" >&2
  ( cd web
    VITE_PROXY_TARGET="http://127.0.0.1:${API_PORT}" \
    nohup npm run dev -- --port "$WEB_PORT" --strictPort --host 127.0.0.1 \
      >"$RUN/web.log" 2>&1 &
    echo $! > "$RUN/web.pid" )
fi

# ── Wait for both, through the proxy path the browser actually uses ─────────
for _ in $(seq 1 60); do
  if curl -sf -o /dev/null "http://127.0.0.1:${API_PORT}/livez" 2>/dev/null \
  && curl -sf -o /dev/null "http://127.0.0.1:${WEB_PORT}/" 2>/dev/null; then
    echo
    echo "  dev stack up -> http://127.0.0.1:${WEB_PORT}"
    echo "  api http://127.0.0.1:${API_PORT}  db $DATABASE_URL"
    echo "  logs $RUN/{api,web}.log   stop: scripts/dev-down.sh"
    exit 0
  fi
  sleep 1
done

echo "dev-up: stack did not come up; tail of logs:" >&2
tail -20 "$RUN/api.log" "$RUN/web.log" 2>/dev/null >&2
exit 1
