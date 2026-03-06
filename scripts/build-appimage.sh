#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GDK_DIR="$ROOT_DIR/.tauri-gdk/2.10.0"

mkdir -p "$GDK_DIR/loaders"
# Some distros don't ship the expected /usr/lib/gdk-pixbuf-2.0 path.
# Pre-generate a local cache file so linuxdeploy-plugin-gtk can proceed.
gdk-pixbuf-query-loaders > "$GDK_DIR/loaders.cache" || true

export PKG_CONFIG_PATH="$ROOT_DIR/scripts/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export NO_STRIP=1
# Prefer system binutils over conda to avoid toolchain mismatches during bundling.
export PATH="/usr/bin:/bin:$PATH"

exec npx tauri build --bundles appimage "$@"
