//! Streaker: a menu bar app tracking GitHub contribution streaks. The `Streaker` binary just
//! calls [`run`].

mod app;
mod app_menu;
mod avatar;
mod command;
mod contribution;
mod icons;
mod platform;
mod preferences;
mod preferences_window;
mod schedule;
mod tray;
mod update;

use std::fs::{self, File};
use std::path::Path;

use anyhow::Context as _;
use directories::ProjectDirs;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use tray_icon::menu::MenuEvent;
use winit::event_loop::EventLoop;

use crate::app::{App, UserEvent};
use crate::avatar::AvatarCache;
use crate::preferences::PreferencesStore;

/// The log file is truncated on startup once it grows beyond this.
const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// Runs the app until it quits.
pub fn run() -> anyhow::Result<()> {
    let dirs = ProjectDirs::from("com", "jamieweavis", platform::APP_NAME)
        .context("could not determine the home directory")?;
    init_logging(dirs.data_local_dir());
    log::info!("Starting Streaker {}", env!("CARGO_PKG_VERSION"));

    platform::init_notifications();

    let mut builder = EventLoop::<UserEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        // Menu bar only: no dock icon until the preferences window opens. The app menu is
        // installed by `app_menu` instead of winit's default.
        builder.with_activation_policy(ActivationPolicy::Accessory).with_default_menu(false);
    }
    let event_loop = builder.build()?;

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let _ = proxy.send_event(UserEvent::Menu(event.id));
    }));

    let mut app = App::new(
        event_loop.create_proxy(),
        PreferencesStore::new(dirs.config_dir().to_owned()),
        AvatarCache::new(dirs.cache_dir().to_owned()),
    );
    event_loop.run_app(&mut app)?;
    log::info!("Exiting");
    Ok(())
}

fn init_logging(log_dir: &Path) {
    let config = ConfigBuilder::new().add_filter_allow_str(env!("CARGO_CRATE_NAME")).build();
    let level = if cfg!(debug_assertions) { LevelFilter::Debug } else { LevelFilter::Info };

    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> =
        vec![TermLogger::new(level, config.clone(), TerminalMode::Mixed, ColorChoice::Auto)];
    let log_path = log_dir.join("streaker.log");
    match open_log_file(&log_path) {
        Ok(file) => loggers.push(WriteLogger::new(level, config, file)),
        Err(e) => eprintln!("Failed to open log file {}: {e}", log_path.display()),
    }
    let _ = CombinedLogger::init(loggers);
}

fn open_log_file(path: &Path) -> std::io::Result<File> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let too_big = fs::metadata(path).is_ok_and(|m| m.len() > MAX_LOG_BYTES);
    File::options().create(true).append(!too_big).write(true).truncate(too_big).open(path)
}
