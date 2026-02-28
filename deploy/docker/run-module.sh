#!/usr/bin/env bash
set -euo pipefail
MODULE="${ZERO_MODULE:-}"
BIN_DIR="/usr/local/bin"

if [[ -z "$MODULE" ]]; then
  echo "ZERO_MODULE not set; selecting first available binary in ${BIN_DIR}"
  shopt -s nullglob
  binaries=("${BIN_DIR}"/*)
  if [ ${#binaries[@]} -eq 0 ]; then
    echo "No binaries found in ${BIN_DIR}"
    exit 1
  fi
  MODULE=$(basename "${binaries[0]}")
fi

echo "Running ZERO module: ${MODULE}"
exec "${BIN_DIR}/${MODULE}" "$@"
    
