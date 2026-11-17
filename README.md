# Omarchy Wallpapers

A small Omarchy Shell bar plugin for a desktop wallpaper gallery and automatic random rotation.

- Click the centered bar icon to open a thumbnail gallery.
- Click a thumbnail to apply it immediately.
- Use the `+` tile to pick images or drag image files onto it.
- Shuffle the rotation queue and see a live countdown to the next change.
- The initial random interval is 5–15 minutes.

## Install

Requirements: Omarchy, Python with Tkinter, Pillow, and Zenity. On a standard Arch Omarchy install, missing system dependencies can be installed with `sudo pacman -S tk python-pillow zenity`.

```sh
git clone https://github.com/6axam/omarchy-wallpapers.git
cd omarchy-wallpapers
./install.sh
```

The installer adds the `maks.wallpapers` bar widget in the center, installs a user systemd service, and keeps settings at `~/.config/omarchy-wallpapers/settings.json`. Add wallpapers to `~/Pictures/Wallpapers` or use the gallery's `+` tile.

Change `min_minutes` and `max_minutes` in `settings.json` to adjust the random interval. The service reads the settings continuously. The gallery can also be launched from a terminal with:

```sh
python ~/.local/share/omarchy-wallpapers/gallery.py
```

Uninstall with `./uninstall.sh`. Settings and wallpaper images are kept.

## License

This plugin is MIT licensed. The bundled TkinterDnD2 files retain their upstream MIT license in `app/vendor/TKINTERDND2-LICENSE`; TkDND is distributed under its included license terms.
