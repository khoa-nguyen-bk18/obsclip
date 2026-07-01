#!/usr/bin/env bash
set -euo pipefail

REPO="khoa-nguyen-bk18/obsclip"
INSTALL_DIR="/Applications/Obsclip.app"
APP_NAME="Obsclip"

info() { printf '==> %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

# --- guards ---
[[ "$(uname -s)" == "Darwin" ]] || die "Obsclip install is supported on macOS and Windows only."

arch="$(uname -m)"
case "$arch" in
  arm64)  target="aarch64" ;;
  x86_64) target="x64" ;;
  *) die "unsupported macOS architecture: $arch" ;;
esac

# --- version ---
version="${OBSCLIP_VERSION:-}"
if [[ -n "$version" ]]; then
  api_url="https://api.github.com/repos/${REPO}/releases/tags/v${version}"
else
  api_url="https://api.github.com/repos/${REPO}/releases/latest"
fi

info "resolving release…"
release_json="$(
  curl -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "User-Agent: obsclip-installer" \
    "$api_url"
)" || die "failed to fetch release metadata from $api_url — is a release published? see https://github.com/${REPO}/releases"

if [[ -z "$version" ]]; then
  version="$(printf '%s' "$release_json" | grep -m1 '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')"
fi
[[ -n "$version" ]] || die "could not determine release version"

info "installing Obsclip v${version} (${target})"

asset_pattern="Obsclip_${version}_${target}.dmg"
download_url="$(printf '%s' "$release_json" | grep -o "https://github.com[^\"]*${asset_pattern}" | head -1)"
[[ -n "$download_url" ]] || die "no asset matching ${asset_pattern} — see https://github.com/${REPO}/releases"

# --- download ---
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/obsclip-install.XXXXXX")"
cleanup() { rm -rf "$work_dir"; }
trap cleanup EXIT

dmg_path="${work_dir}/${asset_pattern}"
info "downloading ${asset_pattern}…"
curl -fL --progress-bar -o "$dmg_path" "$download_url" || die "download failed: $download_url"

# --- quit running app ---
if pgrep -xq "$APP_NAME" 2>/dev/null; then
  info "quitting running ${APP_NAME}…"
  osascript -e "quit app \"${APP_NAME}\"" || true
  sleep 1
  pkill -x "$APP_NAME" 2>/dev/null || true
fi

# --- install ---
mount_point="$(mktemp -d "${TMPDIR:-/tmp}/obsclip-mount.XXXXXX")"
info "mounting DMG…"
hdiutil attach -nobrowse -readonly -mountpoint "$mount_point" "$dmg_path" >/dev/null

if [[ ! -d "${mount_point}/${APP_NAME}.app" ]]; then
  hdiutil detach "$mount_point" >/dev/null || true
  die "DMG does not contain ${APP_NAME}.app"
fi

info "copying to ${INSTALL_DIR}…"
rm -rf "$INSTALL_DIR"
cp -R "${mount_point}/${APP_NAME}.app" "$INSTALL_DIR"
hdiutil detach "$mount_point" >/dev/null

info "clearing quarantine attributes…"
xattr -cr "$INSTALL_DIR"

# --- launch ---
info "launching ${APP_NAME}…"
open -a "$APP_NAME"

info "done — ${APP_NAME} v${version} is installed and running."
