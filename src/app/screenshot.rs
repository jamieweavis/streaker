//! Poses the app for the README screenshot, which `scripts/screenshot.sh` then takes: fixed
//! preferences and stats, with the preferences window beside the open tray menu.

use std::time::Duration;

use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, msg_send, sel};
use objc2_app_kit::NSScreen;
use objc2_foundation::NSPoint;
use winit::dpi::LogicalPosition;
use winit::event_loop::ActiveEventLoop;

use super::{App, UserEvent};
use crate::contribution::GitHubStats;
use crate::preferences::{IconTheme, Preferences, ReminderTime};

/// How long after launch to pose: time for the status item to take its place in the menu bar
/// and for the avatar to load.
const SETTLE: Duration = Duration::from_millis(1500);
/// Points between the preferences window and the left edge of the tray icon.
const WINDOW_GAP: f64 = 60.0;
/// Points between the top of the screen and the preferences window.
const WINDOW_TOP: f64 = 56.0;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGWarpMouseCursorPosition(point: NSPoint) -> i32;
}

impl App {
    /// Replaces the user's preferences and stats with the ones the screenshot shows. Nothing is
    /// saved: the avatar comes from the user's cache, or is fetched by the preferences window.
    pub(super) fn load_screenshot_fixture(&mut self) {
        self.preferences = Preferences {
            username: "jamieweavis".to_owned(),
            launch_at_login: true,
            reminder_enabled: true,
            reminder_time: ReminderTime { hour: 20, minute: 0 },
            icon_theme: IconTheme::Symbol,
        };
        self.avatar = self.avatar_cache.load(&self.preferences.username);
        self.contributions = None;
        self.stats = Some(GitHubStats {
            best_streak: 35,
            current_streak: 32,
            previous_streak: 14,
            is_streak_at_risk: false,
            most_contributions: 108,
            todays_contributions: 3,
            total_contributions: 3109,
        });
        move_pointer_aside();
    }

    pub(super) fn open_for_screenshot(&mut self, event_loop: &ActiveEventLoop) {
        self.open_preferences(event_loop);
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            std::thread::sleep(SETTLE);
            let _ = proxy.send_event(UserEvent::ScreenshotSettled);
        });
    }

    /// Moves the preferences window beside the tray icon and opens the tray menu, then prints
    /// `posed <left> <bottom>`: the window's left and bottom edges in points from the top left
    /// of the screen, for the script to crop the screenshot around.
    pub(super) fn pose_for_screenshot(&self) {
        let (Some(tray), Some(window)) = (&self.tray, &self.preferences_window) else {
            log::error!("Nothing to pose: the tray icon or preferences window is missing");
            return;
        };
        let button = MainThreadMarker::new()
            .zip(tray.icon().ns_status_item())
            .and_then(|(mtm, item)| item.button(mtm));
        let Some((button, icon_frame)) =
            button.and_then(|button| button.window().map(|window| (button, window.frame())))
        else {
            log::error!("Could not find the tray icon in the menu bar");
            return;
        };

        let window = window.window();
        let size = window.outer_size().to_logical::<f64>(window.scale_factor());
        let left = icon_frame.origin.x - WINDOW_GAP - size.width;
        window.set_outer_position(LogicalPosition::new(left, WINDOW_TOP));

        // The click returns once the menu closes, so it's left to the run loop rather than made
        // from inside winit's event handler.
        // SAFETY: the button responds to `performClick:`, which takes an optional sender.
        let _: () = unsafe {
            msg_send![
                &*button,
                performSelector: sel!(performClick:),
                withObject: None::<&AnyObject>,
                afterDelay: 0.2f64
            ]
        };
        println!("posed {left:.0} {:.0}", WINDOW_TOP + size.height);
    }
}

/// Moves the pointer to the left of the screen, away from the window and menu it would
/// otherwise highlight a control or item of.
fn move_pointer_aside() {
    let Some(screen) = MainThreadMarker::new().and_then(NSScreen::mainScreen) else { return };
    let size = screen.frame().size;
    // SAFETY: the point is a plain value, in the global display coordinates the function takes.
    unsafe { CGWarpMouseCursorPosition(NSPoint::new(size.width * 0.1, size.height * 0.5)) };
}
