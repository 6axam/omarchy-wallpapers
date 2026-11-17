import QtQuick
import Quickshell
import Quickshell.Io
import qs.Ui

BarWidget {
    id: root
    moduleName: "maks.wallpapers"
    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight
    Process {
        id: gallery
        command: ["/usr/bin/python", Quickshell.env("HOME") + "/.local/share/omarchy-wallpapers/gallery.py"]
    }
    BarIconButton {
        id: button
        anchors.fill: parent
        bar: root.bar
        text: "󰋩"
        tooltipText: "Wallpapers"
        onPressed: function(b) { if (!gallery.running) gallery.running = true }
    }
}
