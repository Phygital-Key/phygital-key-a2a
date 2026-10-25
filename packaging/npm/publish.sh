#!/usr/bin/env bash
# Publishes the package build.sh made. A version npm already has is skipped,
# so a rerun is safe.
#
#   packaging/npm/publish.sh <out>
set -euo pipefail

[ $# -eq 1 ] || { echo "usage: $0 <out>"; exit 1; }
OUT="$(cd "$1" && pwd)"

name="$(node -p "require('$OUT/package.json').name")"
v="$(node -p "require('$OUT/package.json').version")"
if [ "$(npm view "$name@$v" version 2>/dev/null)" = "$v" ]; then
  echo "$name@$v is already on npm"
else
  (cd "$OUT" && npm publish --access public)
fi
