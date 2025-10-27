#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  cat <<'USAGE'
Usage: scripts/create-admin-user.sh --email you@example.com --password secret [--display-name "Name"]

Environment:
  DATABASE_URL must be set to the target Postgres connection string.
USAGE
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

cargo run \
  --manifest-path "${REPO_ROOT}/api-server/Cargo.toml" \
  --quiet \
  --bin create_user -- \
  "--role" "admin" \
  "$@"
