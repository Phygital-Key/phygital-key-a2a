#!/bin/sh
# shellcheck disable=SC2034 # every script carries the whole PRODUCT block; not all use every value
# Install a Phygital Key product on macOS or Linux (the PRODUCT block below says
# which). Generated from the template in Phygital-Key/phygital-key.
#
#   curl -fsSL https://github.com/<product repo>/releases/latest/download/install.sh | sh
#
# Downloads the latest release for this machine from GitHub, checks that its
# release list is signed by a pinned release key and that the SHA-256 matches, and puts the program in ~/.local/bin (or $PHYGITAL_KEY_BIN_DIR).
# Set PHYGITAL_KEY_VERSION=v0.3.0 (or later: earlier releases have no signed list) to pin a version. Nothing else is touched.
set -eu

# BEGIN PRODUCT
P_NAME="Phygital Key A2A"
P_BIN=phygital-key-a2a
P_ID=a2a
P_REPO=Phygital-Key/phygital-key-a2a
P_PORT=7478
P_VERSION=v0.3.1
# END PRODUCT
REPO="$P_REPO"
BIN_DIR="${PHYGITAL_KEY_BIN_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*" >&2; }
fail() { say "$P_BIN install: $*"; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || fail "needs $1"; }
need curl
need tar
need uname

case "$(uname -s)" in
  Darwin) os=apple-darwin ;;
  Linux) os=unknown-linux-gnu ;;
  *) fail "unsupported system $(uname -s). On Windows use Scoop or the .zip from the Releases page." ;;
esac
case "$(uname -m)" in
  arm64 | aarch64) arch=aarch64 ;;
  x86_64 | amd64) arch=x86_64 ;;
  *) fail "unsupported CPU $(uname -m)" ;;
esac
target="$arch-$os"

# Pinned release keys (P-256), the same block as in the secure installer.
# BEGIN RELEASE KEYS
RELEASE_KEYS='
-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAES9xIi4ZapWVN7JXqSOU+EdoZKKKX
yIUlBGFIVU7oBoMV7ALMx44mXJH5IKCXS3qkKzc8hq+XEgVQHTP9w9gYIw==
-----END PUBLIC KEY-----
-----BEGIN ML-DSA-65 PUBLIC KEY-----
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
-----END ML-DSA-65 PUBLIC KEY-----
'
# END RELEASE KEYS

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

version="${PHYGITAL_KEY_VERSION:-}"
if [ -n "$version" ]; then
  case "$version" in v[0-9]*.[0-9]*.[0-9]*) ;; *) fail "bad version $version" ;; esac
  base="https://github.com/$REPO/releases/download/$version"
else
  base="https://github.com/$REPO/releases/latest/download"
fi

# The release list (file names and SHA-256) must be signed by a pinned release key.
# release2.txt (newer releases) also says when it was signed and until when it is
# trusted; an older release named with PHYGITAL_KEY_VERSION may only have release.txt.
list=release2.txt
if ! curl -q -fsSL "$base/release2.txt" -o "$tmp/list" 2>/dev/null; then
  [ -n "$version" ] || fail "could not download the signed release list"
  list=release.txt
  curl -q -fsSL "$base/release.txt" -o "$tmp/list" || fail "could not download the signed release list"
fi
curl -q -fsSL "$base/$list.sig" -o "$tmp/list.sig" || fail "could not download the release signature"
need openssl
printf '%s\n' "$RELEASE_KEYS" | awk -v dir="$tmp" '
  /-----BEGIN PUBLIC KEY-----/ { n++; f = dir "/key-" n ".pem" }
  f { print > f }
  /-----END PUBLIC KEY-----/ { close(f); f = "" }'
signed=no
for k in "$tmp"/key-*.pem; do
  [ -f "$k" ] || continue
  openssl dgst -sha256 -verify "$k" -signature "$tmp/list.sig" "$tmp/list" >/dev/null 2>&1 && signed=yes
done
[ "$signed" = yes ] || fail "the release list is not signed by a pinned release key. Nothing was installed."
first=5
if [ "$list" = release2.txt ]; then
  [ "$(sed -n 1p "$tmp/list")" = "phygital-key release manifest v2" ] || fail "unknown release list format"
  signed_at="$(sed -n 3p "$tmp/list" | sed -n 's/^signed \([1-9][0-9]*\)$/\1/p')"
  expires="$(sed -n 4p "$tmp/list" | sed -n 's/^expires \([1-9][0-9]*\)$/\1/p')"
  [ -n "$signed_at" ] && [ -n "$expires" ] || fail "bad release list"
  now="$(date +%s)"
  [ "$now" -lt "$expires" ] || fail "the latest signed release list has expired. Check the Releases page: https://github.com/$REPO/releases"
  [ "$signed_at" -le $((now + 86400)) ] || fail "the release list says it was signed in the future (check this computer's clock)"
else
  [ "$(sed -n 1p "$tmp/list")" = "phygital-key release manifest v1" ] || fail "unknown release list format"
  first=3
fi
got="$(sed -n 2p "$tmp/list" | sed -n 's/^version \(v[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)$/\1/p')"
[ -n "$got" ] || fail "bad release list"
[ -z "$version" ] || [ "$got" = "$version" ] || fail "the signed release list is for $got, not $version"
version="$got"
sed -n "$first,\$p" "$tmp/list" > "$tmp/files"

name="$P_BIN-$version-$target"
expected="$(awk -v n="$name.tar.gz" '$2 == n && length($1) == 64 { print $1 }' "$tmp/files")"
[ -n "$expected" ] || fail "the signed release list has no $name.tar.gz"

say "Downloading $name (signed release list checked)"
curl -q -fsSL "https://github.com/$REPO/releases/download/$version/$name.tar.gz" -o "$tmp/$name.tar.gz" || fail "download failed"
if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "$tmp/$name.tar.gz" | cut -d ' ' -f 1)"
else
  actual="$(shasum -a 256 "$tmp/$name.tar.gz" | cut -d ' ' -f 1)"
fi
[ "$expected" = "$actual" ] || fail "the download does not match its signed SHA-256. Nothing was installed."

tar --no-same-owner -xzf "$tmp/$name.tar.gz" -C "$tmp"
mkdir -p "$BIN_DIR"
install -m 0755 "$tmp/$name/$P_BIN" "$BIN_DIR/$P_BIN"
say "Installed $BIN_DIR/$P_BIN ($version)"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) say "Add it to your PATH: export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac
say "Try it: $P_BIN demo"
say "Update later with: $P_BIN update"
