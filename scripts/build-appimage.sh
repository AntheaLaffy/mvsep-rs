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
# Prefer the host Wayland client: the bundled copy can abort WebKit's EGL setup.
export LINUXDEPLOY_EXCLUDED_LIBRARIES=libwayland-client.so.0
# Prefer system binutils over conda to avoid toolchain mismatches during bundling.
export PATH="/usr/bin:/bin:$PATH"

npx tauri build --bundles appimage "$@"
bash "$ROOT_DIR/scripts/check-appimage-compat.sh" "$ROOT_DIR/src-tauri/target/release/bundle/appimage/MVSEP.AppDir"
