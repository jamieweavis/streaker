// Don't open a console window alongside the app on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! The `Streaker` executable (capitalised for the app bundle). The app lives in the `streaker`
//! library.

fn main() -> anyhow::Result<()> {
    streaker::run()
}
