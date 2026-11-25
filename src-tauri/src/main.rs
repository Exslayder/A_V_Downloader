// Always use the Windows GUI subsystem so no console window flashes on launch,
// including debug builds.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    avdock_lib::run()
}
