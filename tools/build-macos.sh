#!/bin/sh
# Local Apple Silicon/Intel engineering preview, built on the target Mac.
# Ad-hoc signatures support local testing; they do not identify a publisher.
set -eu
cd "$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)"

out=${1:-dist/mac}
target=${CARGO_TARGET_DIR:-target}
if [ -e "$out/Sisu-Preview-macos.zip" ]; then
  echo 'Choose a fresh output directory; a preview archive already exists there.' >&2
  exit 1
fi
for tool in cargo cmake ninja codesign; do
  command -v "$tool" >/dev/null || { echo "Missing build tool: $tool" >&2; exit 1; }
done
cargo fmt --all -- --check
cargo test --release --locked --no-default-features --test policy --test import --test cookies
cargo build --release --locked --features packaging --bins
stage=$(mktemp -d /private/tmp/sisu-stage.XXXXXX)
trap 'rm -rf "$stage"' EXIT HUP INT TERM
"$target/release/stage-posix" "$target/release" "$stage"

app="$stage/Sisu Preview.app"
framework="$app/Contents/Frameworks/Chromium Embedded Framework.framework"
# Finder can attach these metadata attributes while staging a visible .app.
# Remove only signing-incompatible metadata from this generated bundle.
# Quarantine and other security attributes are deliberately retained.
xattr -dr com.apple.FinderInfo "$app" 2>/dev/null || true
xattr -dr com.apple.ResourceFork "$app" 2>/dev/null || true
for library in "$framework"/Libraries/*.dylib; do
  [ ! -f "$library" ] || codesign --force --sign - "$library"
done
codesign --force --sign - "$framework"
for helper in "$app"/Contents/Frameworks/*.app; do
  codesign --force --sign - "$helper"
done
codesign --force --sign - "$app"
codesign --verify --deep --strict "$app"
mkdir -p "$out"
ditto -c -k --norsrc --noextattr --keepParent "$app" "$out/Sisu-Preview-macos.zip"
echo "Local preview ready: $out/Sisu-Preview-macos.zip ($(uname -m))"
echo 'CEF reference UI. No Developer ID signing or notarization; not a production release.'
