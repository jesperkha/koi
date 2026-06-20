#!/usr/bin/env bash
#
# Koi language installer. Downloads the latest release and extracts it.
#
# Usage:
#   curl -sL https://raw.githubusercontent.com/jesperkha/koi/main/setup.sh | bash
#
# Options (environment variables):
#   KOI_INSTALL_DIR   Installation directory (default: $HOME/.local/koi)
#
set -euo pipefail

err() { echo "error: $*" >&2; exit 1; }

REPO="jesperkha/koi"
install_dir="${KOI_INSTALL_DIR:-$HOME/.local/koi}"
install_dir="${install_dir/#\~/$HOME}"
install_dir="$(realpath -m "$install_dir")"

echo "Koi Language Installer"
echo "Install directory: $install_dir"
echo ""

# --- Pre-flight checks ---
for cmd in curl tar mkdir chmod; do
    command -v "$cmd" &>/dev/null || err "'$cmd' is required but not found in PATH"
done

# --- Fetch latest release tag ---
echo "Fetching latest release ..."
tag=$(curl -sL -o /dev/null -w '%{url_effective}' \
    "https://github.com/$REPO/releases/latest" \
    | grep -oP '[^/]+$' || true)
[ -z "$tag" ] && err "Could not determine latest release"
echo "Latest release: $tag"

# --- Download tarball ---
archive="koi-${tag}-linux-amd64.tar.gz"
url="https://github.com/$REPO/releases/download/${tag}/${archive}"

echo "Downloading $archive ..."
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
curl -fsSL "$url" -o "$tmpdir/$archive" || err "Download failed. Check that a release exists for $tag."
echo "Downloaded $archive"

# --- Extract to install directory ---
echo "Installing to $install_dir ..."
mkdir -p "$install_dir"
tar -xzf "$tmpdir/$archive" -C "$install_dir" --strip-components=1
chmod +x "$install_dir/bin/koi"
echo "Installed koi to $install_dir"

# --- Done ---
echo ""
echo "Installation complete!"

if [[ ":$PATH:" != *":$install_dir/bin:"* ]]; then
    echo ""
    echo "Add the following to your shell profile to use koi:"
    echo ""
    echo "  export PATH=\"$install_dir/bin:\$PATH\""
    echo ""
fi
