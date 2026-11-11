//! Checks GitHub for a newer release of Streaker.

use anyhow::{Context as _, bail};
use serde::Deserialize;

use crate::contribution::http_agent;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A published release newer than the running version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// The version without the tag's `v` prefix, e.g. `3.2.2`.
    pub version: String,
    pub url: String,
}

#[derive(Deserialize)]
struct LatestRelease {
    tag_name: String,
}

/// Fetches the latest release and returns it if it's newer than the running version. Blocks
/// until done.
pub fn check_for_update() -> anyhow::Result<Option<Release>> {
    let repository = env!("CARGO_PKG_REPOSITORY");
    let Some(repo) = repository.strip_prefix("https://github.com/") else {
        bail!("repository {repository} is not on GitHub");
    };
    let body = http_agent()
        .get(format!("https://api.github.com/repos/{repo}/releases/latest"))
        .header("Accept", "application/vnd.github+json")
        .call()
        .context("failed to fetch the latest release")?
        .into_body()
        .read_to_string()?;
    let latest: LatestRelease = serde_json::from_str(&body).context("failed to parse the latest release")?;
    let version = latest.tag_name.strip_prefix('v').unwrap_or(&latest.tag_name);
    if !is_newer(version, CURRENT_VERSION) {
        return Ok(None);
    }
    Ok(Some(Release {
        version: version.to_owned(),
        url: format!("{repository}/releases/tag/{}", latest.tag_name),
    }))
}

/// Whether `latest` is a higher `major.minor.patch` version than `current`. Versions that don't
/// parse never count as newer.
fn is_newer(latest: &str, current: &str) -> bool {
    parse_version(latest).zip(parse_version(current)).is_some_and(|(latest, current)| latest > current)
}

fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.').map(|part| part.parse().ok());
    let version = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("3.2.2", "3.2.1"));
        assert!(is_newer("3.10.0", "3.9.9"));
        assert!(is_newer("4.0.0", "3.99.99"));
        assert!(!is_newer("3.2.1", "3.2.1"));
        assert!(!is_newer("3.2.0", "3.2.1"));
    }

    #[test]
    fn unparseable_versions_are_not_newer() {
        assert!(!is_newer("3.2", "3.2.1"));
        assert!(!is_newer("3.2.2-beta.1", "3.2.1"));
        assert!(!is_newer("3.2.2.1", "3.2.1"));
        assert!(!is_newer("latest", "3.2.1"));
    }
}
