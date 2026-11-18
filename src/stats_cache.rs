//! The last fetched contributions, kept on disk so the tray shows them straight away at launch
//! and still has them when GitHub can't be reached.

use std::fs;
use std::path::PathBuf;

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::contribution::Contributions;

/// One user's contributions and when they were fetched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedContributions {
    pub username: String,
    pub fetched_at: DateTime<Utc>,
    pub contributions: Contributions,
}

/// Reads and writes [`CachedContributions`] at a fixed path. Holds one user's at a time.
pub struct StatsCache {
    path: PathBuf,
}

impl StatsCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { path: cache_dir.join("contributions.json") }
    }

    /// Loads the cached contributions if they are `username`'s. A missing or unreadable cache is
    /// the same as an empty one: the next fetch replaces it.
    pub fn load(&self, username: &str) -> Option<CachedContributions> {
        let json = match fs::read_to_string(&self.path) {
            Ok(json) => json,
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    log::warn!("Failed to read stats cache {}: {e}", self.path.display());
                }
                return None;
            }
        };
        let cached: CachedContributions = serde_json::from_str(&json)
            .inspect_err(|e| log::warn!("Ignoring invalid stats cache {}: {e}", self.path.display()))
            .ok()?;
        // GitHub usernames are case-insensitive.
        cached.username.eq_ignore_ascii_case(username).then_some(cached)
    }

    /// Saves atomically (write to a temporary file, then rename).
    pub fn save(&self, cached: &CachedContributions) -> anyhow::Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec(cached)?).with_context(|| format!("writing {}", tmp.display()))?;
        fs::rename(&tmp, &self.path).with_context(|| format!("replacing {}", self.path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_for_the_same_username() {
        let dir = std::env::temp_dir().join(format!("streaker-stats-cache-test-{}", std::process::id()));
        let cache = StatsCache::new(dir.clone());
        assert_eq!(cache.load("octocat"), None);

        let cached = CachedContributions {
            username: "octocat".into(),
            fetched_at: DateTime::from_timestamp(1_750_000_000, 0).unwrap(),
            contributions: serde_json::from_str(r#"{"2025-06-01":4,"2025-06-02":0}"#).unwrap(),
        };
        cache.save(&cached).unwrap();
        assert_eq!(cache.load("octocat"), Some(cached.clone()));
        assert_eq!(cache.load("Octocat"), Some(cached));
        assert_eq!(cache.load("someone-else"), None);

        fs::write(dir.join("contributions.json"), "not json").unwrap();
        assert_eq!(cache.load("octocat"), None);
        fs::remove_dir_all(dir).unwrap();
    }
}
