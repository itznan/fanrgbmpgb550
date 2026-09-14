//! CLI help text printer.

pub fn print_help() {
    println!(
        r##"
MSI Motherboard & Gigabyte GPU RGB Controller (CLI Edition)

Usage:
  fanrgb <command> [arguments]

Color values can be specified as:
  - Preset name  : red, green, blue, cyan, magenta, yellow, white, orange, purple, darkpurple, off
  - Hex          : "#RRGGBB" or RRGGBB  (e.g. "#A020F0", "#4B0082", or A020F0)
                   [!] In PowerShell, always quote hex strings with "#" or omit the "#"
  - RGB integers : R G B                 (e.g. 160 32 240)

=== Motherboard Commands ===
  fanrgb status                    Show current active motherboard zones and settings
  fanrgb <preset|hex|r g b>        Set motherboard static color (e.g. fanrgb "#4B0082", fanrgb red)
  fanrgb mode <animation_mode> [color]
                                   Set animation mode (e.g. rainbow, breathing, meteor, flashing, fire)
  fanrgb off                       Turn off motherboard lighting

=== GPU Commands ===
  fanrgb gpu status                Check NVAPI connection and I2C address
  fanrgb gpu <preset|hex|r g b>    Set GPU static color (e.g. fanrgb gpu "#4B0082", fanrgb gpu red)
  fanrgb gpu mode <mode_name>      Set GPU mode (rainbow, breathing, color_cycle, wave, gradient, flash)
  fanrgb gpu off                   Turn off GPU lighting

=== Synchronized Control (Motherboard + GPU) ===
  fanrgb sync <preset|hex|r g b>   Synchronize motherboard and GPU to a color (e.g. fanrgb sync "#4B0082")
  fanrgb bass [color] [--gpu] [--min <0-255>] [--device <name>] [--fmin <hz>] [--fmax <hz>]
                                   Real-time sub-bass audio visualizer (WASAPI loopback)

=== Graphical User Interface (GUI) ===
  fanrgb                           Launch the professional one-page hardware control GUI
  fanrgb gui                       Launch GUI directly
"##
    );
}
