#!/usr/bin/env bash
# 将发布二进制打成 amd64 .deb / Build an amd64 .deb from the release binary
set -euo pipefail

VERSION="${1:?version required}"
BINARY="${2:?binary path required}"
OUT_DIR="${3:?output directory required}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

SAFE_VERSION="${VERSION#v}"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

mkdir -p "$STAGE/DEBIAN" \
  "$STAGE/usr/bin" \
  "$STAGE/usr/share/applications" \
  "$STAGE/usr/share/doc/markdownmonkey"

install -m 0755 "$BINARY" "$STAGE/usr/bin/markdownmonkey"
install -m 0644 "$ROOT/packaging/linux/markdownmonkey.desktop" \
  "$STAGE/usr/share/applications/markdownmonkey.desktop"
install -m 0644 "$ROOT/README.md" "$ROOT/README_EN.md" "$ROOT/LICENSE" \
  "$STAGE/usr/share/doc/markdownmonkey/"

cat > "$STAGE/DEBIAN/control" <<EOF
Package: markdownmonkey
Version: ${SAFE_VERSION}
Section: editors
Priority: optional
Architecture: amd64
Maintainer: MarkdownMonkey Team
Depends: libgtk-3-0, libwebkit2gtk-4.1-0
Description: A modern Markdown editor built with Dioxus
 MarkdownMonkey is a desktop Markdown editor with live preview,
 syntax highlighting, Mermaid diagrams, and KaTeX math.
EOF

mkdir -p "$OUT_DIR"
DEB_NAME="MarkdownMonkey-${SAFE_VERSION}-linux-x64.deb"
dpkg-deb --build "$STAGE" "$OUT_DIR/$DEB_NAME"
echo "$OUT_DIR/$DEB_NAME"
