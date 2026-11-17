#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$HOME/.local/share/omarchy-wallpapers"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/omarchy-wallpapers"
PLUGIN_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/omarchy/plugins/maks.wallpapers"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"

command -v omarchy >/dev/null || { echo "Omarchy is required." >&2; exit 1; }
command -v python >/dev/null || { echo "Python is required." >&2; exit 1; }
python -c 'import tkinter; from PIL import Image' || { echo "Install python-pillow and Tk support first (Arch: sudo pacman -S tk python-pillow)." >&2; exit 1; }
command -v zenity >/dev/null || { echo "Install zenity first (Arch: sudo pacman -S zenity)." >&2; exit 1; }

mkdir -p "$APP_DIR" "$CONFIG_DIR" "$PLUGIN_DIR" "$UNIT_DIR" "$HOME/Pictures/Wallpapers"
cp -a "$ROOT/app/." "$APP_DIR/"
cp -a "$ROOT/plugin/." "$PLUGIN_DIR/"
cp "$ROOT/systemd/random-wallpaper.service" "$UNIT_DIR/random-wallpaper.service"
if [[ ! -f "$CONFIG_DIR/settings.json" ]]; then
  cp "$ROOT/app/settings.json" "$CONFIG_DIR/settings.json"
fi

omarchy plugin enable maks.wallpapers --section center
systemctl --user daemon-reload
systemctl --user enable --now random-wallpaper.service
echo "Installed. Add images to ~/Pictures/Wallpapers or use the + tile in the gallery."
