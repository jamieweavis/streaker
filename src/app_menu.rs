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
    use tray_icon::menu::PredefinedMenuItem;

    use crate::command::EditAction;

    let app_menu = Submenu::with_items(
        "Streaker",
        true,
        &[
            &item(Command::About, "About Streaker", "Cmd+I"),
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

/// Shows the standard macOS About panel, with the homepage as a clickable link in its credits.
/// muda's predefined About item can only pass plain-text credits, so the panel is opened here.
#[cfg(target_os = "macos")]
pub fn show_macos_about_panel() {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{AnyThread, MainThreadMarker};
    use objc2_app_kit::{
        NSAboutPanelOptionApplicationName, NSAboutPanelOptionApplicationVersion, NSAboutPanelOptionCredits,
        NSApplication, NSColor, NSFont, NSFontAttributeName, NSForegroundColorAttributeName,
        NSLinkAttributeName, NSMutableParagraphStyle, NSParagraphStyleAttributeName, NSTextAlignment,
    };
    use objc2_foundation::{NSAttributedString, NSDictionary, NSString, NSURL, ns_string};

    let Some(mtm) = MainThreadMarker::new() else { return };
    let homepage = env!("CARGO_PKG_HOMEPAGE");
    let Some(url) = NSURL::URLWithString(&NSString::from_str(homepage)) else { return };

    let style = NSMutableParagraphStyle::new();
    style.setAlignment(NSTextAlignment::Center);
    let font = NSFont::systemFontOfSize(NSFont::smallSystemFontSize());
    let color = NSColor::linkColor();
    // SAFETY: the attribute keys are paired with values of the types AppKit expects.
    let attributes = unsafe {
        NSDictionary::<NSString, AnyObject>::from_slices(
            &[
                NSLinkAttributeName,
                NSFontAttributeName,
                NSForegroundColorAttributeName,
                NSParagraphStyleAttributeName,
            ],
            &[&*url as &AnyObject, &*font as &AnyObject, &*color as &AnyObject, &*style as &AnyObject],
        )
    };
    // SAFETY: `attributes` holds valid attributed string attributes.
    let credits = unsafe {
        NSAttributedString::initWithString_attributes(
            NSAttributedString::alloc(),
            &NSString::from_str(homepage),
            Some(&attributes),
        )
    };

    let copyright = NSString::from_str(&format!("Copyright © {}", env!("CARGO_PKG_AUTHORS")));
    let name = NSString::from_str("Streaker");
    let version = NSString::from_str(env!("CARGO_PKG_VERSION"));
    // SAFETY: the option keys are paired with values of the types AppKit expects. AppKit has no
    // constant for the copyright option; its key is the string "Copyright".
    let options = unsafe {
        let keys: [&NSString; 4] = [
            NSAboutPanelOptionApplicationName,
            NSAboutPanelOptionApplicationVersion,
            ns_string!("Copyright"),
            NSAboutPanelOptionCredits,
        ];
        let values: [Retained<AnyObject>; 4] = [
            Retained::into_super(Retained::into_super(name)),
            Retained::into_super(Retained::into_super(version)),
            Retained::into_super(Retained::into_super(copyright)),
            Retained::into_super(Retained::into_super(credits)),
        ];
        NSDictionary::from_retained_objects(&keys, &values)
    };
    let app = NSApplication::sharedApplication(mtm);
    // SAFETY: `options` holds valid About panel options.
    unsafe { app.orderFrontStandardAboutPanelWithOptions(&options) };
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
