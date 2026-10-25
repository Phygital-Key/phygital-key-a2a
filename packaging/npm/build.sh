#!/usr/bin/env bash
# Makes the npm package @phygital-key/a2a for one release from its archives: the
# launcher (npx @phygital-key/a2a) and the released program for every target, in
# dist/<os>-<cpu>/.
#
#   packaging/npm/build.sh <version> <dist> <out>
#     version  0.6.2 (no v)
#     dist     folder with phygital-key-a2a-v<version>-<target>.tar.gz / .zip (the release files)
#     out      folder to write the package to; publish.sh publishes it
set -euo pipefail

BIN=phygital-key-a2a
NAME=@phygital-key/a2a
REPO=Phygital-Key/phygital-key-a2a

[ $# -eq 3 ] || { echo "usage: $0 <version> <dist> <out>"; exit 1; }
V="$1"; DIST="$(cd "$2" && pwd)"; mkdir -p "$3"; OUT="$(cd "$3" && pwd)"
[[ "$V" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "bad version $V"; exit 1; }
HERE="$(cd "$(dirname "$0")" && pwd)"; ROOT="$(cd "$HERE/../.." && pwd)"
DESC="$(sed -n 's/^description = "\(.*\)"$/\1/p' "$ROOT/Cargo.toml" | head -1)"
[ -n "$DESC" ] || { echo "no description in Cargo.toml"; exit 1; }

# release target, node's process.platform-process.arch
TARGETS="
x86_64-unknown-linux-gnu  linux-x64
aarch64-unknown-linux-gnu linux-arm64
aarch64-apple-darwin      darwin-arm64
x86_64-apple-darwin       darwin-x64
x86_64-pc-windows-msvc    win32-x64
"

[ -z "$(ls -A "$OUT")" ] || { echo "$OUT is not empty"; exit 1; }
mkdir -p "$OUT/bin"
while read -r target node; do
  [ -n "$target" ] || continue
  mkdir -p "$OUT/dist/$node"
  tmp="$(mktemp -d)"
  if [ "$node" = win32-x64 ]; then
    unzip -q "$DIST/$BIN-v$V-$target.zip" -d "$tmp"
    install -m 0755 "$tmp/$BIN-v$V-$target/$BIN.exe" "$OUT/dist/$node/$BIN.exe"
  else
    tar --no-same-owner -xzf "$DIST/$BIN-v$V-$target.tar.gz" -C "$tmp"
    install -m 0755 "$tmp/$BIN-v$V-$target/$BIN" "$OUT/dist/$node/$BIN"
  fi
  rm -rf "$tmp"
done <<< "$TARGETS"

install -m 0755 "$HERE/launcher.js" "$OUT/bin/$BIN.js"
cp "$ROOT/README.md" "$ROOT/LICENSE" "$ROOT/NOTICE" "$OUT/"
cat > "$OUT/package.json" <<EOF
{
  "name": "$NAME",
  "version": "$V",
  "description": $(printf '%s' "$DESC" | sed 's/\\/\\\\/g; s/"/\\"/g; s/^/"/; s/$/"/'),
  "license": "Apache-2.0",
  "author": "Phygital-Key",
  "homepage": "https://github.com/$REPO",
  "repository": { "type": "git", "url": "git+https://github.com/$REPO.git" },
  "bugs": "https://github.com/$REPO/issues",
  "publishConfig": { "access": "public" },
  "bin": { "$BIN": "bin/$BIN.js" },
  "files": ["bin/", "dist/", "README.md", "LICENSE", "NOTICE"],
  "engines": { "node": ">=18" },
  "os": ["darwin", "linux", "win32"],
  "keywords": ["a2a", "agent2agent", "human-in-the-loop", "approval", "passkey", "webauthn", "ai-agents"]
}
EOF

node -e "JSON.parse(require('fs').readFileSync(process.argv[1]))" "$OUT/package.json"
echo "$NAME@$V in $OUT:"; (cd "$OUT" && find dist -type f | sort)
