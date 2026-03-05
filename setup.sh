#!/usr/bin/env bash
set -euo pipefail

err() { echo "error: $*" >&2; exit 1; }

REPO="jesperkha/koi"
default_dir="$HOME/.local/koi"

echo "Koi Language Installer"
echo ""
echo "Where should Koi be installed?"
read -rp "  Installation directory [$default_dir]: " install_dir
install_dir="${install_dir:-$default_dir}"
install_dir="${install_dir/#\~/$HOME}"
install_dir="$(realpath -m "$install_dir")"

echo ""

# --- Fetch latest release tag ---
echo "Fetching latest release ..."
tag=$(curl -sL -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" | grep -oP '[^/]+$' || true)
[ -z "$tag" ] && err "Could not determine latest release"
echo "Latest release: $tag"

# --- Download tarball ---
archive="koi-${tag}-linux-amd64.tar.gz"
url="https://github.com/$REPO/releases/download/${tag}/${archive}"

echo "Downloading $archive ..."
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
curl -sL "$url" -o "$tmpdir/$archive" || err "Download failed. Check that a release exists for $tag."
echo "Downloaded $archive"

# --- Extract to install directory ---
echo "Installing to $install_dir"
mkdir -p "$install_dir"
tar -xzf "$tmpdir/$archive" -C "$install_dir" --strip-components=1
chmod +x "$install_dir/bin/koi"
echo "Installed koi to $install_dir"

# --- Post-install hint ---
echo ""
echo "Installation complete!"

if [[ ":$PATH:" != *":$install_dir/bin:"* ]]; then
    echo ""
    echo "Add the following to your shell profile to use koi:"
    echo ""
    echo "  export PATH=\"$install_dir/bin:\$PATH\""
    echo ""
fi
