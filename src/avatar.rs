//! GitHub avatars, shown next to the username in the preferences window.

use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::Context as _;
use winit::event_loop::EventLoopProxy;

use crate::app::UserEvent;

/// Edge length in pixels of the fetched avatar: enough for a small avatar on a 2x display.
const SIZE: u32 = 96;

/// Fetches an avatar on a background thread; the result arrives as [`UserEvent::AvatarFetched`].
pub fn fetch_in_background(username: String, proxy: EventLoopProxy<UserEvent>) {
    std::thread::spawn(move || {
        let avatar = fetch(&username);
        // Only fails if the event loop has exited.
        let _ = proxy.send_event(UserEvent::AvatarFetched { username, avatar });
    });
}

/// Fetches and decodes a user's avatar, or `None` if there's no such user or the fetch failed.
/// Blocks until done.
fn fetch(username: &str) -> Option<egui::ColorImage> {
    try_fetch(username).unwrap_or_else(|e| {
        log::warn!("Failed to fetch avatar for {username}: {e:#}");
        None
    })
}

fn try_fetch(username: &str) -> anyhow::Result<Option<egui::ColorImage>> {
    // Redirects to the avatar on avatars.githubusercontent.com.
    let url = format!("https://github.com/{username}.png?size={SIZE}");
    let bytes = match crate::contribution::http_agent().get(url).call() {
        Ok(response) => response.into_body().read_to_vec()?,
        Err(ureq::Error::StatusCode(404)) => return Ok(None),
        Err(e) => return Err(e).context("failed to fetch avatar"),
    };
    let image = image::load_from_memory(&bytes).context("failed to decode avatar")?.into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Ok(Some(egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw())))
}

/// Keeps the saved username's avatar on disk so the preferences window can show it straight
/// away, even after a restart. Holds at most one avatar.
pub struct AvatarCache {
    dir: PathBuf,
}

impl AvatarCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { dir: cache_dir.join("avatars") }
    }

    /// `None` for usernames that can't have an avatar, which also keeps them out of the path.
    fn path(&self, username: &str) -> Option<PathBuf> {
        crate::contribution::is_valid_username(username)
            .then(|| self.dir.join(format!("{}.png", username.to_ascii_lowercase())))
    }

    /// The cached avatar for `username`, if it's the one cached.
    pub fn load(&self, username: &str) -> Option<egui::ColorImage> {
        let path = self.path(username)?;
        if !path.exists() {
            return None;
        }
        match image::open(&path) {
            Ok(image) => {
                let image = image.into_rgba8();
                let size = [image.width() as usize, image.height() as usize];
                Some(egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()))
            }
            Err(e) => {
                log::warn!("Ignoring invalid cached avatar {}: {e}", path.display());
                None
            }
        }
    }

    /// Replaces the cached avatar with `username`'s, or just clears it if there's none.
    pub fn save(&self, username: &str, avatar: Option<&egui::ColorImage>) -> anyhow::Result<()> {
        match fs::remove_dir_all(&self.dir) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                return Err(e).with_context(|| format!("removing {}", self.dir.display()));
            }
            _ => {}
        }
        let (Some(path), Some(avatar)) = (self.path(username), avatar) else { return Ok(()) };

        fs::create_dir_all(&self.dir).with_context(|| format!("creating {}", self.dir.display()))?;
        let pixels = avatar.pixels.iter().flat_map(|pixel| pixel.to_srgba_unmultiplied()).collect();
        let [width, height] = avatar.size.map(|side| side as u32);
        let image = image::RgbaImage::from_raw(width, height, pixels).context("avatar has the wrong size")?;
        image
            .save_with_format(&path, image::ImageFormat::Png)
            .with_context(|| format!("writing {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_keeps_only_the_saved_avatar() {
        let dir = std::env::temp_dir().join(format!("streaker-avatar-test-{}", std::process::id()));
        let cache = AvatarCache::new(dir.clone());
        let avatar = egui::ColorImage::new([2, 3], vec![egui::Color32::from_rgb(10, 20, 30); 6]);

        cache.save("Octocat", Some(&avatar)).unwrap();
        assert_eq!(cache.load("octocat"), Some(avatar.clone()));

        cache.save("hubot", Some(&avatar)).unwrap();
        assert_eq!(cache.load("octocat"), None);
        assert_eq!(cache.load("Hubot"), Some(avatar));

        cache.save("hubot", None).unwrap();
        assert_eq!(cache.load("hubot"), None);
        fs::remove_dir_all(dir).unwrap();
    }
}
