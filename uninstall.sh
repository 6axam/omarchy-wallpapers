#!/usr/bin/env bash
set -euo pipefail

APP_DIR="$HOME/.local/share/omarchy-wallpapers"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
systemctl --user disable --now random-wallpaper.service 2>/dev/null || true
omarchy plugin disable maks.wallpapers 2>/dev/null || true
rm -rf "$CONFIG_HOME/omarchy/plugins/maks.wallpapers"
rm -f "$CONFIG_HOME/systemd/user/random-wallpaper.service"
systemctl --user daemon-reload
rm -rf "$APP_DIR"
echo "Removed the plugin and service. Settings and wallpaper images were kept."
