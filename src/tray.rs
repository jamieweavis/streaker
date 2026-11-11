//! The tray (menu bar) icon and its native menu.

use std::cell::Cell;

use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::command::Command;
use crate::contribution::GitHubStats;
use crate::icons::{self, IconState};
use crate::preferences::IconTheme;

/// Everything the tray displays.
pub struct TrayView<'a> {
    pub username: &'a str,
    pub stats: Option<&'a GitHubStats>,
    pub icon: IconState,
    pub theme: IconTheme,
    pub error: Option<&'a str>,
    /// The version of a newer release, if one is available.
    pub update: Option<&'a str>,
}

/// The tray icon plus handles to the menu items that change, so the menu is updated in place
/// (and stays correct while open) rather than rebuilt.
pub struct Tray {
    icon: TrayIcon,
    /// Kept here rather than handed to tray-icon: see `attach_macos_menu`.
    menu: Menu,
    current_icon: Cell<Option<(IconTheme, IconState)>>,
    /// Shown at the top of the menu, with its separator, only while no username is set.
    set_username: MenuItem,
    set_username_separator: PredefinedMenuItem,
    showing_set_username: Cell<bool>,
    /// Best, current and previous streak, then most, today's and total contributions.
    #[cfg(not(target_os = "macos"))]
    stats: [MenuItem; 6],
    reload: MenuItem,
    open_profile: MenuItem,
    /// Shown above Preferences, only while a newer release is available.
    update_available: MenuItem,
    showing_update_available: Cell<bool>,
    /// The native items behind `stats`, given attributed titles so their values line up.
    #[cfg(target_os = "macos")]
    macos_stats: Vec<objc2::rc::Retained<objc2_app_kit::NSMenuItem>>,
}

fn item(command: Command, text: &str, enabled: bool, accelerator: Option<&str>) -> MenuItem {
    let accelerator = accelerator.map(|a| a.parse().expect("valid accelerator"));
    MenuItem::with_id(command.id(), text, enabled, accelerator)
}

fn label(text: &str) -> MenuItem {
    MenuItem::new(text, false, None)
}

/// Section labels heading each group of stat rows. On macOS they become native section headers.
const SECTIONS: [&str; 2] = ["Streak", "Contributions"];
/// Stat rows are indented under their section label; macOS lays them out natively instead.
#[cfg(not(target_os = "macos"))]
const INDENT: &str = "    ";

impl Tray {
    pub fn new(view: &TrayView) -> anyhow::Result<Self> {
        let set_username = item(Command::OpenPreferences, "Set GitHub username...", true, None);
        let stats: [MenuItem; 6] = std::array::from_fn(|_| label(""));
        let reload = item(Command::Reload, "Reload", false, Some("CmdOrCtrl+R"));
        let open_profile = item(Command::OpenProfile, "Open GitHub Profile...", false, Some("CmdOrCtrl+O"));
        let update_available = item(Command::OpenLatestRelease, "Update Available...", true, None);

        let separator = PredefinedMenuItem::separator;
        let menu = Menu::new();
        menu.append_items(&[
            &label(SECTIONS[0]),
            &stats[0],
            &stats[1],
            &stats[2],
            &separator(),
            &label(SECTIONS[1]),
            &stats[3],
            &stats[4],
            &stats[5],
            &separator(),
            &reload,
            &open_profile,
            &separator(),
            &item(Command::OpenPreferences, "Preferences...", true, Some("CmdOrCtrl+,")),
            &item(Command::About, "About Streaker...", true, None),
            &separator(),
            &item(Command::Quit, "Quit Streaker", true, Some("CmdOrCtrl+Q")),
        ])?;

        let builder = TrayIconBuilder::new().with_tooltip("Streaker");
        #[cfg(not(target_os = "macos"))]
        let builder = builder.with_menu(Box::new(menu.clone()));
        let icon = builder.build()?;
        #[cfg(target_os = "macos")]
        let macos_stats = attach_macos_menu(&icon, &menu);
        let tray = Self {
            icon,
            menu,
            current_icon: Cell::new(None),
            set_username,
            set_username_separator: separator(),
            showing_set_username: Cell::new(false),
            #[cfg(not(target_os = "macos"))]
            stats,
            reload,
            open_profile,
            update_available,
            showing_update_available: Cell::new(false),
            #[cfg(target_os = "macos")]
            macos_stats,
        };
        tray.update(view);
        Ok(tray)
    }

    pub fn update(&self, view: &TrayView) {
        let has_username = !view.username.is_empty();
        if self.showing_set_username.replace(!has_username) == has_username {
            let result = if has_username {
                self.menu
                    .remove(&self.set_username)
                    .and_then(|()| self.menu.remove(&self.set_username_separator))
            } else {
                self.menu.prepend_items(&[&self.set_username, &self.set_username_separator])
            };
            if let Err(e) = result {
                log::warn!("Failed to update the set username menu item: {e}");
            }
        }
        self.reload.set_enabled(has_username);
        self.open_profile.set_enabled(has_username);
        self.update_update_available(view.update);

        let s = view.stats.cloned().unwrap_or_default();
        let at_risk = if s.is_streak_at_risk { "\t(at risk!)" } else { "" };
        let rows = [
            ("Best", s.best_streak.to_string()),
            ("Current", s.current_streak.to_string()),
            ("Previous", format!("{}{at_risk}", s.previous_streak)),
            ("Most", s.most_contributions.to_string()),
            ("Today", s.todays_contributions.to_string()),
            ("Total", s.total_contributions.to_string()),
        ];
        #[cfg(target_os = "macos")]
        {
            let column = macos_value_column(rows.iter().map(|(label, _)| *label));
            for (item, (label, value)) in self.macos_stats.iter().zip(rows) {
                set_macos_stat_title(item, label, &value, column);
            }
        }
        // Windows right-aligns text after a tab, as it does keyboard shortcuts.
        #[cfg(not(target_os = "macos"))]
        for (item, (label, value)) in self.stats.iter().zip(rows) {
            item.set_text(format!("{INDENT}{label}\t{value}"));
        }

        let tooltip = view.error.map_or("Streaker".to_owned(), |error| format!("Streaker: {error}"));
        if let Err(e) = self.icon.set_tooltip(Some(tooltip)) {
            log::warn!("Failed to set tray tooltip: {e}");
        }

        if self.current_icon.replace(Some((view.theme, view.icon))) != Some((view.theme, view.icon)) {
            self.set_icon(view.theme, view.icon);
        }
    }

    fn update_update_available(&self, update: Option<&str>) {
        if let Some(version) = update {
            self.update_available.set_text(format!("Update Available (v{version})..."));
        }
        if self.showing_update_available.replace(update.is_some()) == update.is_some() {
            return;
        }
        let result = if update.is_some() {
            // The Set GitHub username item shares Preferences' id, but always comes before it.
            let preferences = Command::OpenPreferences.id();
            let position = self.menu.items().iter().rposition(|item| *item.id() == preferences);
            self.menu.insert(&self.update_available, position.unwrap_or(0))
        } else {
            self.menu.remove(&self.update_available)
        };
        if let Err(e) = result {
            log::warn!("Failed to update the update available menu item: {e}");
        }
    }

    fn set_icon(&self, theme: IconTheme, state: IconState) {
        let image = Some(icons::tray_icon(theme, state));
        // macOS icons are monochrome template images, tinted by the system to suit the menu bar.
        #[cfg(target_os = "macos")]
        let result = self.icon.set_icon_templated(image).inspect(|_| set_macos_icon_size(&self.icon));
        #[cfg(not(target_os = "macos"))]
        let result = self.icon.set_icon(image);
        if let Err(e) = result {
            log::error!("Failed to set tray icon: {e}");
        }
    }
}

/// tray-icon draws custom icons at one point per pixel, but the macOS icons are @2x artwork
/// (32px for 16pt, as Electron rendered them), so shrink the image to its intended point size.
/// Keeping the full resolution keeps it sharp on Retina displays.
#[cfg(target_os = "macos")]
fn set_macos_icon_size(icon: &TrayIcon) {
    use objc2::MainThreadMarker;
    use objc2_foundation::NSSize;

    let (Some(mtm), Some(item)) = (MainThreadMarker::new(), icon.ns_status_item()) else {
        return;
    };
    let Some(button) = item.button(mtm) else { return };
    let Some(image) = button.image() else { return };
    image.setSize(NSSize::new(icons::MACOS_ICON_POINTS, icons::MACOS_ICON_POINTS));
    // Re-assign so the button lays out for the new size.
    button.setImage(None);
    button.setImage(Some(&image));
}

/// Attaches the menu to the status item natively, so AppKit handles the button itself and
/// supports press-drag-release as well as click-to-open. tray-icon instead overlays the button
/// with a view that opens the menu with a synthesised click on mouse down, which leaves the
/// menu open but ignores the held button. That overlay is hidden so clicks reach the button.
/// Returns the native stat row items.
///
/// The overlay is found by tray-icon's private class name ([`TRAY_ICON_OVERLAY_CLASS`], as of
/// tray-icon 0.26). If an upgrade renames it, the overlay stays and a warning is logged.
#[cfg(target_os = "macos")]
fn attach_macos_menu(icon: &TrayIcon, menu: &Menu) -> Vec<objc2::rc::Retained<objc2_app_kit::NSMenuItem>> {
    use objc2::MainThreadMarker;
    use objc2::rc::Retained;
    use objc2_app_kit::NSMenu;
    use tray_icon::menu::ContextMenu;

    let (Some(mtm), Some(item)) = (MainThreadMarker::new(), icon.ns_status_item()) else {
        return Vec::new();
    };
    // SAFETY: muda returns a valid NSMenu pointer that lives as long as `menu`; retaining it
    // keeps it alive independently.
    let Some(ns_menu) = (unsafe { Retained::retain(menu.ns_menu().cast::<NSMenu>()) }) else {
        return Vec::new();
    };
    let stats = style_macos_sections(&ns_menu, mtm);
    item.setMenu(Some(&ns_menu));
    if let Some(button) = item.button(mtm) {
        let mut hidden = false;
        for view in button.subviews() {
            if view.class().name().to_bytes() == TRAY_ICON_OVERLAY_CLASS.as_bytes() {
                view.setHidden(true);
                hidden = true;
            }
        }
        if !hidden {
            log::warn!(
                "tray-icon's {TRAY_ICON_OVERLAY_CLASS} overlay not found; press-drag-release won't open the menu"
            );
        }
    }
    stats
}

/// The class of the view tray-icon overlays the status item button with: see `attach_macos_menu`.
#[cfg(target_os = "macos")]
const TRAY_ICON_OVERLAY_CLASS: &str = "TaoTrayTarget";

/// Replaces the plain section labels with native section headers (small, grey and bold, as in
/// the system's own menus). Section headers need macOS 14, so older versions keep the labels and
/// indent the stat rows beneath them instead. Returns the stat rows, in menu order.
#[cfg(target_os = "macos")]
fn style_macos_sections(
    ns_menu: &objc2_app_kit::NSMenu,
    mtm: objc2::MainThreadMarker,
) -> Vec<objc2::rc::Retained<objc2_app_kit::NSMenuItem>> {
    use objc2_app_kit::NSMenuItem;
    use objc2_foundation::NSString;

    let rows_per_section = 3;
    let mut stats = Vec::new();
    for title in SECTIONS {
        let title = NSString::from_str(title);
        let index = ns_menu.indexOfItemWithTitle(&title);
        if index < 0 {
            continue;
        }
        let has_headers = objc2::available!(macos = 14.0);
        if has_headers {
            ns_menu.removeItemAtIndex(index);
            ns_menu.insertItem_atIndex(&NSMenuItem::sectionHeaderWithTitle(&title, mtm), index);
        }
        for row in index + 1..=index + rows_per_section {
            if let Some(item) = ns_menu.itemAtIndex(row) {
                if !has_headers {
                    item.setIndentationLevel(1);
                }
                stats.push(item);
            }
        }
    }
    stats
}

/// Where stat values start, in points from the start of the title: three spaces after the
/// longest label, so the values line up in a column just past the labels.
#[cfg(target_os = "macos")]
fn macos_value_column<'a>(labels: impl Iterator<Item = &'a str>) -> f64 {
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSFont, NSFontAttributeName, NSStringDrawing};
    use objc2_foundation::{NSDictionary, NSString};

    let font = NSFont::menuFontOfSize(0.0);
    // SAFETY: the font attribute is paired with an NSFont.
    let attributes = unsafe {
        NSDictionary::<NSString, AnyObject>::from_slices(&[NSFontAttributeName], &[&*font as &AnyObject])
    };
    // SAFETY: `attributes` holds valid string drawing attributes.
    let width = |text: &str| unsafe { NSString::from_str(text).sizeWithAttributes(Some(&attributes)).width };
    labels.map(width).fold(0.0, f64::max) + width("   ")
}

/// Sets a stat row's title to its label, then its value starting at `column`. Text after a tab
/// in the value (the at-risk note) follows it after a few spaces.
#[cfg(target_os = "macos")]
fn set_macos_stat_title(item: &objc2_app_kit::NSMenuItem, label: &str, value: &str, column: f64) {
    use objc2::AnyThread;
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{
        NSFont, NSFontAttributeName, NSMutableParagraphStyle, NSParagraphStyleAttributeName, NSTextAlignment,
        NSTextTab,
    };
    use objc2_foundation::{NSArray, NSAttributedString, NSDictionary, NSString};

    let tab = |alignment, location| {
        // SAFETY: an empty options dictionary is valid.
        unsafe {
            NSTextTab::initWithTextAlignment_location_options(
                NSTextTab::alloc(),
                alignment,
                location,
                &NSDictionary::new(),
            )
        }
    };
    let style = NSMutableParagraphStyle::new();
    style.setTabStops(Some(&NSArray::from_retained_slice(&[tab(NSTextAlignment::Left, column)])));
    // Attributed titles don't inherit the menu font, so set it explicitly.
    let font = NSFont::menuFontOfSize(0.0);
    // SAFETY: the attribute keys are paired with values of the types AppKit expects.
    let attributes = unsafe {
        NSDictionary::<NSString, AnyObject>::from_slices(
            &[NSFontAttributeName, NSParagraphStyleAttributeName],
            &[&*font as &AnyObject, &*style as &AnyObject],
        )
    };
    let text = NSString::from_str(&format!("{label}\t{}", value.replace('\t', "   ")));
    // SAFETY: `attributes` holds valid attributed string attributes.
    let title = unsafe {
        NSAttributedString::initWithString_attributes(NSAttributedString::alloc(), &text, Some(&attributes))
    };
    item.setAttributedTitle(Some(&title));
}
