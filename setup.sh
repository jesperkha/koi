#!/usr/bin/env bash
set -euo pipefail

bold=$'\033[1m'
green=$'\033[0;32m'
cyan=$'\033[0;36m'
red=$'\033[0;31m'
reset=$'\033[0m'

info()  { printf "${cyan}::${reset} %s\n" "$*"; }
ok()    { printf "${green}✓${reset} %s\n" "$*"; }
err()   { printf "${red}error:${reset} %s\n" "$*" >&2; exit 1; }

REPO="jesperkha/koi"
default_dir="$HOME/.local/koi"

printf "\n${bold}Koi Language Installer${reset}\n\n"
printf "Where should Koi be installed?\n"
read -rp "  Installation directory [$default_dir]: " install_dir
install_dir="${install_dir:-$default_dir}"
install_dir="${install_dir/#\~/$HOME}"
install_dir="$(realpath -m "$install_dir")"

printf "\n"

# --- Fetch latest release tag ---
info "Fetching latest release …"
tag=$(curl -sL -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" | grep -oP '[^/]+$')
[ -z "$tag" ] && err "Could not determine latest release"
ok "Latest release: $tag"

# --- Download tarball ---
archive="koi-${tag}-linux-amd64.tar.gz"
url="https://github.com/$REPO/releases/download/${tag}/${archive}"

info "Downloading $archive …"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
curl -sL "$url" -o "$tmpdir/$archive" || err "Download failed. Check that a release exists for $tag."
ok "Downloaded $archive"

# --- Extract to install directory ---
info "Installing to ${bold}${install_dir}${reset}"
mkdir -p "$install_dir"
tar -xzf "$tmpdir/$archive" -C "$install_dir" --strip-components=1
chmod +x "$install_dir/bin/koi"
ok "Installed koi to $install_dir"

# --- Post-install hint ---
printf "\n${green}${bold}Installation complete!${reset}\n\n"

if [[ ":$PATH:" != *":$install_dir/bin:"* ]]; then
    printf "Add the following to your shell profile to use ${bold}koi${reset}:\n\n"
    printf "  export PATH=\"%s/bin:\$PATH\"\n\n" "$install_dir"
fi
