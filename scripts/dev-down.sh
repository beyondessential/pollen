#!/usr/bin/env bash
# Stop the detached dev stack started by dev-up.sh (leaves the cluster's data).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUN="$ROOT/.dev-stack"
PGDIR="$HOME/.cache/pollen-dev-pg"

for name in api web; do
  pid_file="$RUN/$name.pid"
  if [ -f "$pid_file" ]; then
    pid="$(cat "$pid_file")"
    if kill -0 "$pid" 2>/dev/null; then
      # npm spawns vite as a child; take the group down with it.
      kill -- -"$(ps -o pgid= -p "$pid" | tr -d ' ')" 2>/dev/null || kill "$pid" 2>/dev/null
      echo "dev-down: stopped $name (pid $pid)"
    fi
    rm -f "$pid_file"
  fi
done

if pg_ctl -D "$PGDIR" status >/dev/null 2>&1; then
  pg_ctl -D "$PGDIR" -m fast stop >/dev/null 2>&1 && echo "dev-down: stopped postgres"
fi
