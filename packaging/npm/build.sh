#!/usr/bin/env bash
# Makes the npm packages for one release from its archives:
#   phygital-key-a2a                  the launcher (npx phygital-key-a2a), lists the five below
#   phygital-key-a2a-<os>-<cpu>       the released program for one kind of computer
#
#   packaging/npm/build.sh <version> <dist> <out>
#     version  0.6.2 (no v)
#     dist     folder with phygital-key-a2a-v<version>-<target>.tar.gz / .zip (the release files)
#     out      folder to write the packages to (one subfolder each); publish.sh publishes them
set -euo pipefail

BIN=phygital-key-a2a
REPO=Phygital-Key/phygital-key-a2a

[ $# -eq 3 ] || { echo "usage: $0 <version> <dist> <out>"; exit 1; }
V="$1"; DIST="$(cd "$2" && pwd)"; mkdir -p "$3"; OUT="$(cd "$3" && pwd)"
[[ "$V" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "bad version $V"; exit 1; }
HERE="$(cd "$(dirname "$0")" && pwd)"; ROOT="$(cd "$HERE/../.." && pwd)"
DESC="$(sed -n 's/^description = "\(.*\)"$/\1/p' "$ROOT/Cargo.toml" | head -1)"
[ -n "$DESC" ] || { echo "no description in Cargo.toml"; exit 1; }

# release target, npm os, npm cpu, libc ("-" when npm should not check it)
TARGETS="
x86_64-unknown-linux-gnu  linux  x64   glibc
aarch64-unknown-linux-gnu linux  arm64 glibc
aarch64-apple-darwin      darwin arm64 -
x86_64-apple-darwin       darwin x64   -
x86_64-pc-windows-msvc    win32  x64   -
"

common() { # the fields every package shares
  cat <<EOF
  "version": "$V",
  "license": "Apache-2.0",
  "author": "Phygital-Key",
  "homepage": "https://github.com/$REPO",
  "repository": { "type": "git", "url": "git+https://github.com/$REPO.git" },
  "bugs": "https://github.com/$REPO/issues",
EOF
}

optional=""
while read -r target os cpu libc; do
  [ -n "$target" ] || continue
  name="$BIN-$os-$cpu"
  dir="$OUT/$name"
  rm -rf "$dir"; mkdir -p "$dir/bin"
  tmp="$(mktemp -d)"
  if [ "$os" = win32 ]; then
    unzip -q "$DIST/$BIN-v$V-$target.zip" -d "$tmp"
    install -m 0755 "$tmp/$BIN-v$V-$target/$BIN.exe" "$dir/bin/$BIN.exe"
  else
    tar --no-same-owner -xzf "$DIST/$BIN-v$V-$target.tar.gz" -C "$tmp"
    install -m 0755 "$tmp/$BIN-v$V-$target/$BIN" "$dir/bin/$BIN"
  fi
  rm -rf "$tmp"
  cp "$ROOT/LICENSE" "$ROOT/NOTICE" "$dir/"
  libc_line=""; [ "$libc" = - ] || libc_line="  \"libc\": [\"$libc\"],"
  {
    echo "{"
    echo "  \"name\": \"$name\","
    echo "  \"description\": \"The $BIN program for $os $cpu. Install $BIN, not this package.\","
    common
    echo "  \"os\": [\"$os\"],"
    echo "  \"cpu\": [\"$cpu\"],"
    [ -z "$libc_line" ] || echo "$libc_line"
    echo "  \"files\": [\"bin/\", \"LICENSE\", \"NOTICE\"]"
    echo "}"
  } > "$dir/package.json"
  printf '# %s\n\nThe `%s` program for %s %s, from the [v%s release](https://github.com/%s/releases/tag/v%s). Install [`%s`](https://www.npmjs.com/package/%s) instead: it picks this package on its own.\n' \
    "$name" "$BIN" "$os" "$cpu" "$V" "$REPO" "$V" "$BIN" "$BIN" > "$dir/README.md"
  optional="$optional    \"$name\": \"$V\",
"
done <<< "$TARGETS"

dir="$OUT/$BIN"
rm -rf "$dir"; mkdir -p "$dir/bin"
install -m 0755 "$HERE/launcher.js" "$dir/bin/$BIN.js"
cp "$ROOT/README.md" "$ROOT/LICENSE" "$ROOT/NOTICE" "$dir/"
{
  echo "{"
  echo "  \"name\": \"$BIN\","
  echo "  \"description\": $(printf '%s' "$DESC" | sed 's/\\/\\\\/g; s/"/\\"/g; s/^/"/; s/$/"/'),"
  common
  echo "  \"bin\": { \"$BIN\": \"bin/$BIN.js\" },"
  echo "  \"files\": [\"bin/\", \"README.md\", \"LICENSE\", \"NOTICE\"],"
  echo "  \"engines\": { \"node\": \">=18\" },"
  echo "  \"keywords\": [\"a2a\", \"agent2agent\", \"human-in-the-loop\", \"approval\", \"passkey\", \"webauthn\", \"ai-agents\"],"
  echo "  \"optionalDependencies\": {"
  printf '%s' "$optional" | sed '$ s/,$//'
  echo "  }"
  echo "}"
} > "$dir/package.json"

for p in "$OUT"/*/package.json; do node -e "JSON.parse(require('fs').readFileSync(process.argv[1]))" "$p"; done
echo "npm packages for v$V in $OUT:"; ls "$OUT"
