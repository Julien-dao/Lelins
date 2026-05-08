#!/usr/bin/env bash
# Generate the `latest.json` manifest consumed by tauri-plugin-updater.
#
# Usage: generate-latest-json.sh <version> <artifacts_dir>
#
# Where <artifacts_dir> contains the per-target subfolders produced by
# `actions/upload-artifact@v4` in `release.yml` :
#
#   artifacts/
#   ├── andrea-aarch64-apple-darwin/
#   │   ├── ANDREA_${VERSION}_aarch64.app.tar.gz
#   │   └── ANDREA_${VERSION}_aarch64.app.tar.gz.sig
#   ├── andrea-x86_64-apple-darwin/
#   │   └── ...
#   └── andrea-x86_64-pc-windows-msvc/
#       └── ANDREA_${VERSION}_x64-setup.msi.zip(.sig)
#
# The signature files are produced by Tauri's signing step using the
# Minisign key in TAURI_SIGNING_PRIVATE_KEY. Their content is base64-encoded
# Minisign signatures (single-line); we embed them verbatim.

set -euo pipefail

VERSION="${1:?usage: generate-latest-json.sh <version> <artifacts_dir>}"
ARTIFACTS_DIR="${2:?usage: generate-latest-json.sh <version> <artifacts_dir>}"
NOTES="${NOTES:-Version ${VERSION}}"
PUB_DATE="$(date -u +"%Y-%m-%dT%H:%M:%S.000Z")"
RELEASE_BASE="https://github.com/Julien-dao/Lelins/releases/download/${VERSION}"

read_sig() {
  local sig_file="$1"
  if [[ -f "$sig_file" ]]; then
    tr -d '\n' < "$sig_file"
  else
    echo ""
  fi
}

# Locate the signed bundles, preferring the `.app.tar.gz` (Mac) /
# `.msi.zip` (Win) variants which are the formats Tauri's updater
# downloads transparently.

mac_arm_archive="$(ls "${ARTIFACTS_DIR}"/andrea-aarch64-apple-darwin/*.app.tar.gz 2>/dev/null | head -1 || true)"
mac_arm_sig="$(ls "${ARTIFACTS_DIR}"/andrea-aarch64-apple-darwin/*.app.tar.gz.sig 2>/dev/null | head -1 || true)"
mac_x64_archive="$(ls "${ARTIFACTS_DIR}"/andrea-x86_64-apple-darwin/*.app.tar.gz 2>/dev/null | head -1 || true)"
mac_x64_sig="$(ls "${ARTIFACTS_DIR}"/andrea-x86_64-apple-darwin/*.app.tar.gz.sig 2>/dev/null | head -1 || true)"
win_archive="$(ls "${ARTIFACTS_DIR}"/andrea-x86_64-pc-windows-msvc/*.msi.zip 2>/dev/null | head -1 || true)"
win_sig="$(ls "${ARTIFACTS_DIR}"/andrea-x86_64-pc-windows-msvc/*.msi.zip.sig 2>/dev/null | head -1 || true)"

# `jq -n` builds the JSON safely even when one of the platform paths is
# missing (handy during partial test releases).
jq -n \
  --arg version       "${VERSION#v}" \
  --arg notes         "$NOTES" \
  --arg pub_date      "$PUB_DATE" \
  --arg mac_arm_url   "${mac_arm_archive:+${RELEASE_BASE}/$(basename "$mac_arm_archive")}" \
  --arg mac_arm_sig   "$(read_sig "$mac_arm_sig")" \
  --arg mac_x64_url   "${mac_x64_archive:+${RELEASE_BASE}/$(basename "$mac_x64_archive")}" \
  --arg mac_x64_sig   "$(read_sig "$mac_x64_sig")" \
  --arg win_url       "${win_archive:+${RELEASE_BASE}/$(basename "$win_archive")}" \
  --arg win_sig       "$(read_sig "$win_sig")" '
{
  version:  $version,
  notes:    $notes,
  pub_date: $pub_date,
  platforms: (
    {}
    | (if $mac_arm_url != "" then . + { "darwin-aarch64": { signature: $mac_arm_sig, url: $mac_arm_url } } else . end)
    | (if $mac_x64_url != "" then . + { "darwin-x86_64":  { signature: $mac_x64_sig, url: $mac_x64_url } } else . end)
    | (if $win_url     != "" then . + { "windows-x86_64": { signature: $win_sig,     url: $win_url     } } else . end)
  )
}'
