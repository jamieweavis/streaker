//! Small platform integrations: dock visibility, launch at login and notifications.

use anyhow::Context as _;
use auto_launch::{AutoLaunch, AutoLaunchBuilder, MacOSLaunchMode};

pub const APP_NAME: &str = "Streaker";
#[cfg(target_os = "macos")]
pub const BUNDLE_IDENTIFIER: &str = "com.jamieweavis.streaker";

/// Shows or hides the macOS dock icon. Streaker is an accessory (menu bar only) app, but shows
/// in the dock and app switcher while the preferences window is open. No-op elsewhere.
pub fn set_dock_visible(visible: bool) {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

        let Some(mtm) = MainThreadMarker::new() else {
            log::error!("set_dock_visible called off the main thread");
            return;
        };
        let app = NSApplication::sharedApplication(mtm);
        let policy = if visible {
            NSApplicationActivationPolicy::Regular
        } else {
            NSApplicationActivationPolicy::Accessory
        };
        app.setActivationPolicy(policy);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = visible;
}

/// Brings the app to the front. Menu bar apps are never the active app on macOS, so focusing a
/// window alone would leave it behind the frontmost app.
pub fn activate_app() {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSApplication;

        if let Some(mtm) = MainThreadMarker::new() {
            // `activate` requires macOS 14; this works everywhere.
            #[allow(deprecated)]
            NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
        }
    }
}

/// The macOS mode is ignored on other platforms.
fn auto_launch(mode: MacOSLaunchMode) -> anyhow::Result<AutoLaunch> {
    let exe = std::env::current_exe().context("locating executable")?;
    let exe = exe.to_str().context("executable path is not valid UTF-8")?;
    Ok(AutoLaunchBuilder::new()
        .set_app_name(APP_NAME)
        .set_app_path(exe)
        .set_macos_launch_mode(mode)
        .build()?)
}

/// Registers or unregisters Streaker to launch at login.
///
/// On macOS 13+ this uses `SMAppService`, so Streaker shows under "Open at Login" in System
/// Settings and the registration follows the app bundle. Older macOS (and other platforms) fall
/// back to a login item that enabling always (re)writes, so it points at the currently running
/// executable even if the app has moved.
pub fn set_launch_at_login(enabled: bool) -> anyhow::Result<()> {
    let launch_agent = auto_launch(MacOSLaunchMode::LaunchAgent)?;

    #[cfg(target_os = "macos")]
    if let Some(app_service) = app_service()? {
        // Remove the LaunchAgent written by earlier versions, or Streaker would launch twice.
        if launch_agent.is_enabled()? {
            launch_agent.disable()?;
            log::info!("Removed legacy launch at login LaunchAgent");
        }
        if app_service.is_enabled()? == enabled {
            return Ok(());
        }
        return toggle_launch_at_login(&app_service, enabled);
    }

    toggle_launch_at_login(&launch_agent, enabled)
}

/// Reconciles launch at login with the system at startup and returns whether it is enabled.
///
/// On macOS 13+ the user can turn the login item off (or back on) in System Settings, so the
/// system's status wins over `preferred`, except when migrating from the legacy LaunchAgent.
/// Elsewhere `preferred` is applied, re-pointing the login item at this executable.
pub fn sync_launch_at_login(preferred: bool) -> anyhow::Result<bool> {
    #[cfg(target_os = "macos")]
    if let Some(app_service) = app_service()?
        && !auto_launch(MacOSLaunchMode::LaunchAgent)?.is_enabled()?
    {
        return Ok(app_service.is_enabled()?);
    }

    set_launch_at_login(preferred)?;
    Ok(preferred)
}

/// The `SMAppService` login item, or `None` before macOS 13 where it is unavailable.
#[cfg(target_os = "macos")]
fn app_service() -> anyhow::Result<Option<AutoLaunch>> {
    match auto_launch(MacOSLaunchMode::SMAppService) {
        Ok(app_service) => Ok(Some(app_service)),
        Err(e) if matches!(e.downcast_ref(), Some(auto_launch::Error::UnsupportedOS)) => Ok(None),
        Err(e) => Err(e),
    }
}

fn toggle_launch_at_login(auto_launch: &AutoLaunch, enabled: bool) -> anyhow::Result<()> {
    if enabled {
        auto_launch.enable()?;
    } else if auto_launch.is_enabled()? {
        auto_launch.disable()?;
    } else {
        return Ok(());
    }
    log::info!("Launch at login {}", if enabled { "enabled" } else { "disabled" });
    Ok(())
}

/// Must be called once at startup before showing notifications.
pub fn init_notifications() {
    #[cfg(target_os = "macos")]
    if let Err(e) = notify_rust::set_application(BUNDLE_IDENTIFIER) {
        // Expected when running unbundled (e.g. `cargo run`); notifications then appear to come
        // from the default application.
        log::warn!("Could not attribute notifications to {BUNDLE_IDENTIFIER}: {e}");
    }
}

/// Shows a desktop notification on a background thread (delivery can block on some platforms).
pub fn show_notification(title: String, body: String) {
    std::thread::spawn(move || {
        let result = notify_rust::Notification::new().appname(APP_NAME).summary(&title).body(&body).show();
        if let Err(e) = result {
            log::error!("Failed to show notification: {e}");
        }
    });
}
