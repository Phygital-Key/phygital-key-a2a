#!/usr/bin/env bash
# Publishes the packages build.sh made: the five programs first, then the launcher
# that lists them, so `npx @phygital-key/a2a` never points at a missing version.
# A version npm already has is skipped, so a rerun finishes the rest.
#
#   packaging/npm/publish.sh <out>
set -euo pipefail

PKG=a2a
[ $# -eq 1 ] || { echo "usage: $0 <out>"; exit 1; }
OUT="$(cd "$1" && pwd)"

publish() {
  local dir="$1" name v
  name="$(node -p "require('$dir/package.json').name")"
  v="$(node -p "require('$dir/package.json').version")"
  if [ "$(npm view "$name@$v" version 2>/dev/null)" = "$v" ]; then
    echo "$name@$v is already on npm"
  else
    (cd "$dir" && npm publish --access public)
  fi
}

for dir in "$OUT/$PKG"-*/; do publish "${dir%/}"; done
publish "$OUT/$PKG"
