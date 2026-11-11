//! Tray icons for each theme and contribution state, embedded at compile time.

use tray_icon::Icon;

use crate::contribution::GitHubStats;
use crate::preferences::IconTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconState {
    /// No contributions today (or stats not loaded yet).
    Pending,
    /// Contributed today.
    Contributed,
    /// Contributed today and the current streak is the best streak.
    Streaking,
    /// No username set, or the stats could not be fetched.
    Error,
}

impl IconState {
    pub fn for_stats(stats: &GitHubStats) -> Self {
        if stats.current_streak == 0 {
            IconState::Pending
        } else if stats.current_streak == stats.best_streak {
            IconState::Streaking
        } else {
            IconState::Contributed
        }
    }
}

/// Point size of the macOS menu bar icons, which are 32px @2x images.
#[cfg(target_os = "macos")]
pub const MACOS_ICON_POINTS: f64 = 16.0;

#[cfg(target_os = "macos")]
macro_rules! icon_path {
    ($theme:literal, $state:literal) => {
        concat!("../assets/macos/", $theme, "/", $state, "@2x.png")
    };
}

#[cfg(windows)]
macro_rules! icon_path {
    ($theme:literal, $state:literal) => {
        concat!("../assets/windows/", $theme, "/", $state, ".ico")
    };
}

#[cfg(not(any(target_os = "macos", windows)))]
macro_rules! icon_path {
    ($theme:literal, $state:literal) => {
        concat!("../assets/linux/", $theme, "/", $state, ".png")
    };
}

/// A theme's encoded icons, indexed by [`IconState`].
macro_rules! theme_icons {
    ($theme:literal) => {
        [
            include_bytes!(icon_path!($theme, "pending")).as_slice(),
            include_bytes!(icon_path!($theme, "contributed")),
            include_bytes!(icon_path!($theme, "streaking")),
            include_bytes!(icon_path!($theme, "error")),
        ]
    };
}

fn encoded(theme: IconTheme, state: IconState) -> &'static [u8] {
    let icons: [&'static [u8]; 4] = match theme {
        IconTheme::Flame => theme_icons!("flame"),
        IconTheme::Tile => theme_icons!("tile"),
        IconTheme::Octocat => theme_icons!("octocat"),
    };
    icons[state as usize]
}

/// A theme's encoded pending, contributed and streaking icons, for showing in the preferences
/// window. They are single-color silhouettes, so only their alpha is meaningful.
pub fn preview_images(theme: IconTheme) -> [&'static [u8]; 3] {
    [IconState::Pending, IconState::Contributed, IconState::Streaking].map(|state| encoded(theme, state))
}

pub fn tray_icon(theme: IconTheme, state: IconState) -> Icon {
    let image = image::load_from_memory(encoded(theme, state))
        .expect("embedded tray icon is a valid image")
        .into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).expect("embedded tray icon has valid dimensions")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_icons_decode() {
        for theme in IconTheme::ALL {
            for state in [IconState::Pending, IconState::Contributed, IconState::Streaking, IconState::Error]
            {
                tray_icon(theme, state);
            }
        }
    }

    /// The Windows icons only decode on Windows builds, so check them on every platform in CI.
    #[test]
    fn windows_icons_decode() {
        for theme in ["flame", "tile", "octocat"] {
            for state in ["pending", "contributed", "streaking", "error"] {
                let path = format!("{}/assets/windows/{theme}/{state}.ico", env!("CARGO_MANIFEST_DIR"));
                let bytes = std::fs::read(&path).unwrap();
                image::load_from_memory(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_icons_are_2x() {
        for theme in IconTheme::ALL {
            for state in [IconState::Pending, IconState::Contributed, IconState::Streaking, IconState::Error]
            {
                let image = image::load_from_memory(encoded(theme, state)).unwrap();
                assert_eq!(f64::from(image.width()), MACOS_ICON_POINTS * 2.0);
                assert_eq!(f64::from(image.height()), MACOS_ICON_POINTS * 2.0);
            }
        }
    }

    #[test]
    fn state_for_stats() {
        let stats =
            |current_streak, best_streak| GitHubStats { current_streak, best_streak, ..Default::default() };
        assert_eq!(IconState::for_stats(&stats(0, 0)), IconState::Pending);
        assert_eq!(IconState::for_stats(&stats(0, 5)), IconState::Pending);
        assert_eq!(IconState::for_stats(&stats(2, 5)), IconState::Contributed);
        assert_eq!(IconState::for_stats(&stats(5, 5)), IconState::Streaking);
    }
}
