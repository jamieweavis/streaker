//! User preferences and their persistence as JSON in the platform config directory.

use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub username: String,
    pub launch_at_login: bool,
    pub reminder_enabled: bool,
    pub reminder_time: ReminderTime,
    pub icon_theme: IconTheme,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            username: String::new(),
            launch_at_login: false,
            reminder_enabled: false,
            reminder_time: ReminderTime { hour: 20, minute: 0 },
            icon_theme: IconTheme::Flame,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IconTheme {
    Flame,
    Tile,
    Octocat,
}

impl IconTheme {
    pub const ALL: [IconTheme; 3] = [IconTheme::Flame, IconTheme::Tile, IconTheme::Octocat];

    pub fn label(self) -> &'static str {
        match self {
            IconTheme::Flame => "Flame",
            IconTheme::Tile => "Tile",
            IconTheme::Octocat => "Octocat",
        }
    }
}

/// A local time of day, stored as `"HH:MM"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ReminderTime {
    pub hour: u8,
    pub minute: u8,
}

impl fmt::Display for ReminderTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

impl FromStr for ReminderTime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("invalid reminder time \"{s}\", expected HH:MM");
        let (hour, minute) = s.split_once(':').ok_or_else(invalid)?;
        let hour: u8 = hour.parse().map_err(|_| invalid())?;
        let minute: u8 = minute.parse().map_err(|_| invalid())?;
        if hour > 23 || minute > 59 {
            return Err(invalid());
        }
        Ok(Self { hour, minute })
    }
}

impl TryFrom<String> for ReminderTime {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<ReminderTime> for String {
    fn from(time: ReminderTime) -> Self {
        time.to_string()
    }
}

/// Reads and writes [`Preferences`] at a fixed path.
pub struct PreferencesStore {
    path: PathBuf,
}

impl PreferencesStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { path: config_dir.join("preferences.json") }
    }

    /// Loads preferences, falling back to defaults if the file is missing or unreadable.
    pub fn load(&self) -> Preferences {
        match fs::read_to_string(&self.path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
                log::error!("Ignoring invalid preferences file {}: {e}", self.path.display());
                Preferences::default()
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Preferences::default(),
            Err(e) => {
                log::error!("Failed to read preferences file {}: {e}", self.path.display());
                Preferences::default()
            }
        }
    }

    /// Saves preferences atomically (write to a temporary file, then rename).
    pub fn save(&self, preferences: &Preferences) -> anyhow::Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(preferences)?)
            .with_context(|| format!("writing {}", tmp.display()))?;
        fs::rename(&tmp, &self.path).with_context(|| format!("replacing {}", self.path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reminder_time_round_trips() {
        let time: ReminderTime = "07:05".parse().unwrap();
        assert_eq!(time, ReminderTime { hour: 7, minute: 5 });
        assert_eq!(time.to_string(), "07:05");
        assert!("24:00".parse::<ReminderTime>().is_err());
        assert!("12:60".parse::<ReminderTime>().is_err());
        assert!("noon".parse::<ReminderTime>().is_err());
    }

    #[test]
    fn deserializes_partial_json_with_defaults() {
        let prefs: Preferences =
            serde_json::from_str(r#"{"username":"octocat","iconTheme":"tile"}"#).unwrap();
        assert_eq!(prefs.username, "octocat");
        assert_eq!(prefs.icon_theme, IconTheme::Tile);
        assert_eq!(prefs.reminder_time, ReminderTime { hour: 20, minute: 0 });
    }

    #[test]
    fn store_saves_and_loads() {
        let dir = std::env::temp_dir().join(format!("streaker-test-{}", std::process::id()));
        let store = PreferencesStore::new(dir.clone());
        assert_eq!(store.load(), Preferences::default());

        let prefs = Preferences {
            username: "octocat".into(),
            reminder_enabled: true,
            reminder_time: ReminderTime { hour: 9, minute: 30 },
            ..Preferences::default()
        };
        store.save(&prefs).unwrap();
        assert_eq!(store.load(), prefs);
        fs::remove_dir_all(dir).unwrap();
    }
}
