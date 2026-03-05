#!/usr/bin/env bash
set -euo pipefail

# --- Colors & helpers ---
bold=$'\033[1m'
green=$'\033[0;32m'
cyan=$'\033[0;36m'
red=$'\033[0;31m'
reset=$'\033[0m'

info()  { printf "${cyan}::${reset} %s\n" "$*"; }
ok()    { printf "${green}✓${reset} %s\n" "$*"; }
err()   { printf "${red}error:${reset} %s\n" "$*" >&2; }

# --- Pre-flight checks ---
for cmd in cargo cp mkdir; do
    if ! command -v "$cmd" &>/dev/null; then
        err "'$cmd' is required but not found in PATH"
        exit 1
    fi
done

# --- Prompt for install directory ---
default_dir="$HOME/.local/koi"

printf "\n${bold}Koi Language Installer${reset}\n\n"
printf "Where should Koi be installed?\n"
read -rp "  Installation directory [$default_dir]: " install_dir
install_dir="${install_dir:-$default_dir}"

# Expand ~ if the user typed it
install_dir="${install_dir/#\~/$HOME}"

# Resolve to absolute path
install_dir="$(realpath -m "$install_dir")"

printf "\n"
info "Installing to ${bold}${install_dir}${reset}"

# --- Create directory structure ---
info "Creating directory structure …"
mkdir -p "$install_dir"/{lib,external,bin}
ok "Created $install_dir/{lib,external,bin}"

# --- Copy runtime files ---
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

info "Copying runtime files …"
cp "$script_dir/lib/entry.s" "$install_dir/lib/entry.s"
ok "Copied lib/entry.s"

# --- Build release binary ---
info "Building koi (release) — this may take a moment …"
cargo build --release --manifest-path "$script_dir/Cargo.toml"
ok "Build succeeded"

# --- Install binary ---
info "Installing binary …"
cp "$script_dir/target/release/koi" "$install_dir/bin/koi"
chmod +x "$install_dir/bin/koi"
ok "Installed koi binary to $install_dir/bin/koi"

# --- Post-install hint ---
printf "\n${green}${bold}Installation complete!${reset}\n\n"

# Check if the bin dir is already on PATH
if [[ ":$PATH:" != *":$install_dir/bin:"* ]]; then
    printf "Add the following to your shell profile to use ${bold}koi${reset}:\n\n"
    printf "  export PATH=\"%s/bin:\$PATH\"\n\n" "$install_dir"
fi
