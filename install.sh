#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$HOME/.local/share/omarchy-wallpapers"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/omarchy-wallpapers"
PLUGIN_DIR="$HOME/.config/omarchy/plugins/maks.wallpapers"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
command -v omarchy >/dev/null || { echo 'Omarchy is required.' >&2; exit 1; }
command -v cargo >/dev/null || { echo 'Rust and Cargo are required.' >&2; exit 1; }
pkg-config --exists gtk+-3.0 || { echo 'GTK 3 development files are required (Arch: sudo pacman -S gtk3).' >&2; exit 1; }
(cd "$ROOT" && cargo build --release --locked)
mkdir -p "$APP_DIR" "$CONFIG_DIR" "$PLUGIN_DIR" "$UNIT_DIR" "$HOME/Pictures/Wallpapers"
install -m 755 "$ROOT/target/release/omarchy-wallpapers" "$APP_DIR/omarchy-wallpapers.new"
mv "$APP_DIR/omarchy-wallpapers.new" "$APP_DIR/omarchy-wallpapers"
cp -a "$ROOT/plugin/." "$PLUGIN_DIR/"
cp "$ROOT/systemd/random-wallpaper.service" "$UNIT_DIR/random-wallpaper.service"
if [[ ! -f "$CONFIG_DIR/settings.json" ]]; then cp "$ROOT/app/settings.json" "$CONFIG_DIR/settings.json"; fi
systemctl --user daemon-reload
systemctl --user enable random-wallpaper.service
systemctl --user restart random-wallpaper.service
omarchy plugin enable maks.wallpapers --section center
echo 'Installed. Add images through the gallery or ~/Pictures/Wallpapers.'
