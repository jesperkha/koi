#!/usr/bin/env bash
#
# Koi language installer (from source). Builds the compiler and installs it
# along with runtime files to the given directory.
#
# Usage:
#   bash install.sh [INSTALL_DIR]
#
# If INSTALL_DIR is not provided, defaults to $HOME/.local/koi.
#
set -euo pipefail

err() { echo "error: $*" >&2; exit 1; }

# --- Pre-flight checks ---
for cmd in cargo cp mkdir; do
    command -v "$cmd" &>/dev/null || err "'$cmd' is required but not found in PATH"
done

# --- Determine install directory ---
default_dir="$HOME/.local/koi"
install_dir="${1:-$default_dir}"
install_dir="${install_dir/#\~/$HOME}"
install_dir="$(realpath -m "$install_dir")"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Koi Language Installer (from source)"
echo "Install directory: $install_dir"
echo ""

# --- Create directory structure ---
echo "Creating directory structure ..."
mkdir -p "$install_dir"/{lib,external,bin}

# --- Copy runtime files ---
echo "Copying runtime files ..."
cp "$script_dir/lib/entry.s" "$install_dir/lib/entry.s"

# --- Build release binary ---
echo "Building koi (release) - this may take a moment ..."
cargo build --release --manifest-path "$script_dir/Cargo.toml"
echo "Build succeeded"

# --- Install binary ---
echo "Installing binary ..."
cp "$script_dir/target/release/koi" "$install_dir/bin/koi"
chmod +x "$install_dir/bin/koi"
echo "Installed koi binary to $install_dir/bin/koi"

# --- Post-install hint ---
echo ""
echo "Installation complete!"

# Check if the bin dir is already on PATH
if [[ ":$PATH:" != *":$install_dir/bin:"* ]]; then
    echo ""
    echo "Add the following to your shell profile to use koi:"
    echo ""
    echo "  export PATH=\"$install_dir/bin:\$PATH\""
    echo ""
fi
