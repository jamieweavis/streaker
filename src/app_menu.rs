//! Native application menus.
//!
//! - macOS: the global menu bar (app, Edit and Window menus), installed once at startup. It is
//!   only visible while the preferences window is open, as Streaker is otherwise an accessory
//!   (menu bar only) app.
//! - Windows: a File menu attached to the preferences window.
//! - Linux: none. Native menu bars there require GTK windows, and the preferences window's
//!   shortcuts are handled by egui instead.

#[cfg(any(target_os = "macos", windows))]
use tray_icon::menu::{Menu, MenuItem, Submenu};

#[cfg(any(target_os = "macos", windows))]
use crate::command::Command;

#[cfg(any(target_os = "macos", windows))]
fn item(command: Command, text: &str, accelerator: &str) -> MenuItem {
    MenuItem::with_id(command.id(), text, true, Some(accelerator.parse().expect("valid accelerator")))
}

/// Installs the macOS application menu. The returned menu must be kept alive.
#[cfg(target_os = "macos")]
pub fn install_macos_menu() -> anyhow::Result<Menu> {
    use tray_icon::menu::{AboutMetadata, PredefinedMenuItem};

    use crate::command::EditAction;

    let about = AboutMetadata {
        name: Some("Streaker".into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        copyright: Some(format!("Copyright © {}", env!("CARGO_PKG_AUTHORS"))),
        website: Some(env!("CARGO_PKG_HOMEPAGE").into()),
        ..Default::default()
    };

    let app_menu = Submenu::with_items(
        "Streaker",
        true,
        &[
            &PredefinedMenuItem::about(Some("About Streaker"), Some(about)),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &item(Command::Quit, "Quit Streaker", "Cmd+Q"),
        ],
    )?;

    let edit = |action, text, accelerator| item(Command::Edit(action), text, accelerator);
    let edit_menu = Submenu::with_items(
        "Edit",
        true,
        &[
            &edit(EditAction::Undo, "Undo", "Cmd+Z"),
            &edit(EditAction::Redo, "Redo", "Shift+Cmd+Z"),
            &PredefinedMenuItem::separator(),
            &edit(EditAction::Cut, "Cut", "Cmd+X"),
            &edit(EditAction::Copy, "Copy", "Cmd+C"),
            &edit(EditAction::Paste, "Paste", "Cmd+V"),
            &edit(EditAction::SelectAll, "Select All", "Cmd+A"),
        ],
    )?;

    let window_menu = Submenu::with_items("Window", true, &[&item(Command::CloseWindow, "Close", "Cmd+W")])?;

    let menu = Menu::with_items(&[&app_menu, &edit_menu, &window_menu])?;
    menu.init_for_nsapp();
    window_menu.set_as_windows_menu_for_nsapp();
    Ok(menu)
}

/// Attaches a File menu to a Windows window. The returned menu must be kept alive as long as
/// the window.
#[cfg(windows)]
pub fn attach_windows_menu(window: &winit::window::Window) -> anyhow::Result<Menu> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let file_menu = Submenu::with_items("&File", true, &[&item(Command::CloseWindow, "&Close", "Ctrl+W")])?;
    let menu = Menu::with_items(&[&file_menu])?;

    let RawWindowHandle::Win32(handle) = window.window_handle()?.as_raw() else {
        anyhow::bail!("expected a Win32 window handle");
    };
    // SAFETY: the handle belongs to a live window owned by the caller, which keeps the menu
    // alive for as long as the window exists.
    unsafe { menu.init_for_hwnd(handle.hwnd.get())? };
    Ok(menu)
}
