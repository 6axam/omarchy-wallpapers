# Omarchy Wallpapers

An Omarchy Shell bar widget with a Rust wallpaper gallery and random rotation service.

- Click the centered bar icon to open the gallery; click a picture to set it immediately.
- Add pictures through the `+` tile using the file picker or drag and drop.
- The active wallpaper is outlined; a countdown shows the next automatic change.
- Shuffle rearranges the queue and chooses a new random delay. The default is 5–15 minutes.

## Install

Requires Omarchy, GTK 3, Rust and Cargo. On Arch/Omarchy, install missing build dependencies with `sudo pacman -S rust gtk3 pkgconf`.

```sh
git clone https://github.com/6axam/omarchy-wallpapers.git
cd omarchy-wallpapers
./install.sh
```

The installer compiles the Rust program, enables the `maks.wallpapers` bar widget, and starts a user systemd service. Upgrading preserves existing `~/.config/omarchy-wallpapers/settings.json` and images. Add images through the gallery or to `~/Pictures/Wallpapers`. The QML widget is the minimal adapter required by Omarchy Shell.

Edit `~/.config/omarchy-wallpapers/settings.json` to change `folder`, `min_minutes`, and `max_minutes`. The service reads the settings while running. Open the gallery from a terminal with:

```sh
~/.local/share/omarchy-wallpapers/omarchy-wallpapers gui
```

Uninstall with `./uninstall.sh`; settings and wallpaper images are kept. On Wayland, GTK 3's native file chooser uses the desktop portal where available.

## License

MIT. The application's gallery and rotation service are implemented in Rust; GTK and Omarchy Shell provide the desktop interface.
