#!/usr/bin/env bash
set -euo pipefail

APPDIR="${1:?usage: check-appimage-compat.sh APPDIR}"
if [[ ! -d "$APPDIR/usr" ]]; then
  echo "AppDir not found: $APPDIR" >&2
  exit 1
fi

bundled_wayland_client="$(find "$APPDIR/usr" -name 'libwayland-client.so*' -print -quit)"
if [[ -n "$bundled_wayland_client" ]]; then
  echo "AppImage bundles a Wayland client that can crash WebKit EGL: $bundled_wayland_client" >&2
  exit 1
fi
