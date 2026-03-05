#!/usr/bin/env bash
set -euo pipefail

err() { echo "error: $*" >&2; exit 1; }

# --- Pre-flight checks ---
for cmd in cargo cp mkdir; do
    if ! command -v "$cmd" &>/dev/null; then
        err "'$cmd' is required but not found in PATH"
    fi
done

# --- Determine install directory ---
default_dir="$HOME/.local/koi"

if [[ $# -ge 1 ]]; then
    install_dir="$1"
else
    echo "Koi Language Installer"
    echo ""
    echo "Where should Koi be installed?"
    read -rp "  Installation directory [$default_dir]: " install_dir
    install_dir="${install_dir:-$default_dir}"
fi

# Expand ~ if the user typed it
install_dir="${install_dir/#\~/$HOME}"

# Resolve to absolute path
install_dir="$(realpath -m "$install_dir")"

echo ""
echo "Installing to $install_dir"

# --- Create directory structure ---
echo "Creating directory structure ..."
mkdir -p "$install_dir"/{lib,external,bin}
echo "Created $install_dir/{lib,external,bin}"

# --- Copy runtime files ---
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Copying runtime files ..."
cp "$script_dir/lib/entry.s" "$install_dir/lib/entry.s"
echo "Copied lib/entry.s"

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
