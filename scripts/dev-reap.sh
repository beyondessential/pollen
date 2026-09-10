#!/usr/bin/env bash
# Reap Vite servers left behind by Playwright runs (its fixture spawns one per
# worker and orphans them on teardown).
#
# The dev stack's own Vite also runs with `--port`, so matching on that alone
# kills it too. Key on the port instead: the dev stack owns WEB_PORT, the e2e
# fixture uses random high ones.
set -uo pipefail

WEB_PORT="${WEB_PORT:-8091}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

pids=$(ps -eo pid=,comm=,command= | awk -v me=$$ -v keep="$WEB_PORT" -v root="$ROOT" '
  $1 != me && $2 ~ /node$/ && index($0, root "/web/node_modules/.bin/vite") \
    && $0 ~ /--port/ && $0 !~ ("--port " keep) { print $1 }')

if [ -z "$pids" ]; then
  echo "dev-reap: nothing to reap"
  exit 0
fi
echo "dev-reap: killing $(echo "$pids" | wc -l | tr -d ' ') orphaned vite server(s)"
echo "$pids" | xargs kill -9 2>/dev/null
