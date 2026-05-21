// Prevents a console window from owning the app process on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    fixtext_lib::run()
}
