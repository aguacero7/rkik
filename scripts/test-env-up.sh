#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${ROOT_DIR}/dev/test-env/docker-compose.yml"
COMPOSE_BIN="${COMPOSE_BIN:-docker compose}"

echo "[rkik] Starting test environment..."
${COMPOSE_BIN} -f "${COMPOSE_FILE}" up -d --build "$@"
