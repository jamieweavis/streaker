//! Application state and the winit event loop handler.

use std::time::Instant;

use tray_icon::menu::MenuId;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::WindowId;

use crate::avatar::AvatarCache;
use crate::command::Command;
use crate::contribution::{FetchError, GitHubStats};
use crate::icons::IconState;
use crate::platform;
use crate::preferences::{Preferences, PreferencesStore};
use crate::preferences_window::{PreferencesWindow, WindowAction};
use crate::schedule::{self, FetchSchedule, Reminder};
use crate::tray::{Tray, TrayView};
use crate::update::Release;

const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
/// Ticks are a minute apart, so a longer gap between them means the computer was asleep.
const WAKE_GAP: chrono::TimeDelta = chrono::TimeDelta::minutes(2);
/// How often to check for a newer release, and how long to wait after a failed check.
const UPDATE_CHECK_INTERVAL: chrono::TimeDelta = chrono::TimeDelta::days(1);
const UPDATE_CHECK_RETRY: chrono::TimeDelta = chrono::TimeDelta::hours(1);

#[derive(Debug)]
pub enum UserEvent {
    /// A native menu item was clicked.
    Menu(MenuId),
    /// A background stats fetch finished.
    StatsFetched { username: String, result: Result<GitHubStats, FetchError> },
    /// A background avatar fetch for the preferences window finished.
    AvatarFetched { username: String, avatar: Option<egui::ColorImage> },
    /// A background check for a newer release finished.
    UpdateChecked(anyhow::Result<Option<Release>>),
    /// A screen reader or other assistive technology interacted with a window.
    AccessKit(egui_winit::accesskit_winit::Event),
}

impl From<egui_winit::accesskit_winit::Event> for UserEvent {
    fn from(event: egui_winit::accesskit_winit::Event) -> Self {
        UserEvent::AccessKit(event)
    }
}

pub struct App {
    proxy: EventLoopProxy<UserEvent>,
    store: PreferencesStore,
    preferences: Preferences,

    stats: Option<GitHubStats>,
    stats_fetched_at: Option<chrono::DateTime<chrono::Local>>,
    fetch_error: Option<String>,
    fetch_in_flight: bool,
    refetch_when_done: bool,
    fetch_schedule: FetchSchedule,

    /// A newer release than the running version, if one has been found.
    update: Option<Release>,
    /// When to next check for a newer release; `None` checks at the next opportunity.
    next_update_check: Option<chrono::DateTime<chrono::Utc>>,

    tray: Option<Tray>,
    #[cfg(target_os = "macos")]
    _app_menu: Option<tray_icon::menu::Menu>,
    preferences_window: Option<PreferencesWindow>,
    /// The saved username's avatar, cached when preferences are saved so reopening the window
    /// shows it straight away.
    avatar: Option<egui::ColorImage>,
    avatar_cache: AvatarCache,

    next_tick: Instant,
    last_tick: chrono::DateTime<chrono::Local>,
    reminder: Reminder,
}

impl App {
    pub fn new(proxy: EventLoopProxy<UserEvent>, store: PreferencesStore, avatar_cache: AvatarCache) -> Self {
        let preferences = store.load();
        let avatar = avatar_cache.load(&preferences.username);
        Self {
            proxy,
            store,
            preferences,
            stats: None,
            stats_fetched_at: None,
            fetch_error: None,
            fetch_in_flight: false,
            refetch_when_done: false,
            fetch_schedule: FetchSchedule::default(),
            update: None,
            next_update_check: None,
            tray: None,
            #[cfg(target_os = "macos")]
            _app_menu: None,
            preferences_window: None,
            avatar,
            avatar_cache,
            next_tick: Instant::now(),
            last_tick: chrono::Local::now(),
            reminder: Reminder::default(),
        }
    }

    /// One-time setup once the event loop is running (required for tray icons and menus).
    fn start(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_os = "macos")]
        match crate::app_menu::install_macos_menu() {
            Ok(menu) => self._app_menu = Some(menu),
            Err(e) => log::error!("Failed to install application menu: {e:#}"),
        }

        match Tray::new(&self.tray_view()) {
            Ok(tray) => self.tray = Some(tray),
            Err(e) => {
                log::error!("Failed to create tray icon: {e:#}");
                event_loop.exit();
                return;
            }
        }

        self.sync_launch_at_login();

        self.fetch_stats();
        self.check_for_update();
        self.next_tick = schedule::next_minute(&chrono::Local::now());

        // Most likely a fresh install.
        if self.preferences.username.is_empty() {
            self.open_preferences(event_loop);
        }
    }

    /// Runs every wall-clock minute.
    fn tick(&mut self) {
        let now = chrono::Local::now();
        log::debug!("Tick at {now}");
        self.next_tick = schedule::next_minute(&now);
        if now - std::mem::replace(&mut self.last_tick, now) > WAKE_GAP {
            log::info!("Woke from sleep, refreshing stats");
            self.fetch_schedule.fetch_now();
        }

        self.check_reminder(&now);

        if !self.fetch_in_flight && self.fetch_schedule.is_due(now.to_utc()) {
            self.fetch_stats();
        }
        self.check_for_update();
    }

    /// Shows the daily reminder if it's due and the streak is at risk. Until there are stats fresh
    /// enough to decide, it fetches them and is checked again when they arrive.
    fn check_reminder(&mut self, now: &chrono::DateTime<chrono::Local>) {
        let time = self.preferences.reminder_time;
        if !self.preferences.reminder_enabled || !self.reminder.is_pending(now, time) {
            return;
        }
        let latest_fetch_failed = self.fetch_error.is_some();
        let stats = self
            .stats
            .as_ref()
            .zip(self.stats_fetched_at)
            .filter(|(_, fetched_at)| {
                schedule::can_decide_reminder(fetched_at, now, time, latest_fetch_failed)
            })
            .map(|(stats, _)| stats);
        if stats.is_none() && !latest_fetch_failed {
            self.fetch_schedule.fetch_now();
        }
        if let Some((title, body)) = self.reminder.check(now, time, stats) {
            log::info!("Showing reminder notification");
            platform::show_notification(title, body);
        }
    }

    fn tray_view(&self) -> TrayView<'_> {
        let username = self.preferences.username.as_str();
        let error =
            if username.is_empty() { Some("Set your GitHub username") } else { self.fetch_error.as_deref() };
        let icon = match (&self.stats, error) {
            (_, Some(_)) => IconState::Error,
            (Some(stats), None) => IconState::for_stats(stats),
            (None, None) => IconState::Pending,
        };
        TrayView {
            username,
            stats: self.stats.as_ref(),
            icon,
            theme: self.preferences.icon_theme,
            error,
            update: self.update.as_ref().map(|release| release.version.as_str()),
        }
    }

    fn update_tray(&self) {
        if let Some(tray) = &self.tray {
            tray.update(&self.tray_view());
        }
    }

    /// Fetches stats now, on a background thread; the result arrives as [`UserEvent::StatsFetched`].
    /// Ticks refetch them on [`FetchSchedule`].
    fn fetch_stats(&mut self) {
        if self.preferences.username.is_empty() {
            self.update_tray();
            return;
        }
        if self.fetch_in_flight {
            self.refetch_when_done = true;
            return;
        }
        self.fetch_in_flight = true;

        let username = self.preferences.username.clone();
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let result = crate::contribution::fetch_github_stats(&username).map_err(FetchError::from);
            // Only fails if the event loop has exited.
            let _ = proxy.send_event(UserEvent::StatsFetched { username, result });
        });
    }

    fn on_stats_fetched(&mut self, username: String, result: Result<GitHubStats, FetchError>) {
        self.fetch_in_flight = false;

        // Ignore results for a username that has since been changed.
        if username == self.preferences.username {
            match result {
                Ok(stats) => {
                    self.fetch_schedule.succeeded(chrono::Utc::now());
                    log::debug!("Fetched contribution stats for {username}: {stats:?}");
                    self.stats = Some(stats);
                    self.stats_fetched_at = Some(chrono::Local::now());
                    self.fetch_error = None;
                }
                Err(e) => {
                    self.fetch_schedule.failed(chrono::Utc::now(), e.retry_after);
                    log::warn!("Failed to fetch contribution stats for {username}: {}", e.message);
                    self.fetch_error = Some(e.message);
                }
            }
            self.update_tray();
            self.check_reminder(&chrono::Local::now());
        } else {
            self.refetch_when_done = true;
        }

        if std::mem::take(&mut self.refetch_when_done) {
            self.fetch_stats();
        }
    }

    /// Checks for a newer release on a background thread, if a check is due; the result arrives
    /// as [`UserEvent::UpdateChecked`].
    fn check_for_update(&mut self) {
        let now = chrono::Utc::now();
        if self.next_update_check.is_some_and(|due| now < due) {
            return;
        }
        // Also stops another check starting while this one is in flight.
        self.next_update_check = Some(now + UPDATE_CHECK_INTERVAL);

        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let result = crate::update::check_for_update();
            // Only fails if the event loop has exited.
            let _ = proxy.send_event(UserEvent::UpdateChecked(result));
        });
    }

    fn on_update_checked(&mut self, result: anyhow::Result<Option<Release>>) {
        match result {
            Ok(Some(release)) => {
                log::info!("Streaker {} is available", release.version);
                self.update = Some(release);
            }
            Ok(None) => log::debug!("Streaker is up to date"),
            Err(e) => {
                log::warn!("Failed to check for updates: {e:#}");
                self.next_update_check = Some(chrono::Utc::now() + UPDATE_CHECK_RETRY);
            }
        }
        self.update_tray();
    }

    fn save_preferences(&mut self, preferences: Preferences, avatar: Option<egui::ColorImage>) {
        log::info!("Saving preferences: {preferences:?}");
        let previous = std::mem::replace(&mut self.preferences, preferences);
        if let Err(e) = self.avatar_cache.save(&self.preferences.username, avatar.as_ref()) {
            log::error!("Failed to cache avatar: {e:#}");
        }
        self.avatar = avatar;

        if self.preferences.launch_at_login != previous.launch_at_login
            && let Err(e) = platform::set_launch_at_login(self.preferences.launch_at_login)
        {
            // Keep the saved preference matching the system, and say why the toggle didn't stick.
            log::error!("Failed to update launch at login: {e:#}");
            self.preferences.launch_at_login = previous.launch_at_login;
            platform::show_notification("Couldn't change launch at login".to_owned(), format!("{e:#}"));
        }
        if let Err(e) = self.store.save(&self.preferences) {
            log::error!("Failed to save preferences: {e:#}");
        }

        if self.preferences.username != previous.username {
            self.stats = None;
            self.stats_fetched_at = None;
            self.fetch_error = None;
        }
        self.update_tray();
        self.fetch_stats();
    }

    /// Reconciles launch at login with the system: picks up changes made in System Settings,
    /// keeps the login item pointing at this executable and migrates the legacy LaunchAgent.
    /// Skipped for debug builds so `cargo run` doesn't take over the login item.
    fn sync_launch_at_login(&mut self) {
        if cfg!(debug_assertions) {
            return;
        }
        match platform::sync_launch_at_login(self.preferences.launch_at_login) {
            Ok(enabled) if enabled != self.preferences.launch_at_login => {
                log::info!("Launch at login changed outside Streaker: {enabled}");
                self.preferences.launch_at_login = enabled;
                if let Err(e) = self.store.save(&self.preferences) {
                    log::error!("Failed to save preferences: {e:#}");
                }
            }
            Ok(_) => {}
            Err(e) => log::error!("Failed to update launch at login: {e:#}"),
        }
    }

    fn open_preferences(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.preferences_window {
            platform::activate_app();
            window.focus();
            return;
        }

        // Pick up changes made in System Settings since launch before showing the toggle.
        self.sync_launch_at_login();
        platform::set_dock_visible(true);
        platform::activate_app();
        match PreferencesWindow::open(event_loop, &self.preferences, self.avatar.as_ref(), self.proxy.clone())
        {
            Ok(window) => self.preferences_window = Some(window),
            Err(e) => {
                log::error!("Failed to open preferences window: {e:#}");
                platform::set_dock_visible(false);
            }
        }
    }

    fn close_preferences(&mut self) {
        if self.preferences_window.take().is_some() {
            platform::set_dock_visible(false);
        }
    }

    fn run_command(&mut self, event_loop: &ActiveEventLoop, command: Command) {
        let open_url = |url: &str| {
            if let Err(e) = open::that_detached(url) {
                log::error!("Failed to open {url}: {e}");
            }
        };

        match command {
            Command::OpenPreferences => self.open_preferences(event_loop),
            Command::Reload => self.fetch_stats(),
            Command::OpenProfile => open_url(&format!("https://github.com/{}", self.preferences.username)),
            Command::OpenLatestRelease => {
                if let Some(release) = &self.update {
                    open_url(&release.url);
                }
            }
            Command::About => open_url(HOMEPAGE),
            Command::Quit => event_loop.exit(),
            Command::CloseWindow => self.close_preferences(),
            Command::Edit(action) => {
                if let Some(window) = &mut self.preferences_window {
                    window.perform_edit(action);
                }
            }
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            self.start(event_loop);
        } else if Instant::now() >= self.next_tick {
            self.tick();
        }
    }

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Menu(id) => match Command::from_id(&id) {
                Some(command) => self.run_command(event_loop, command),
                None => log::warn!("Unknown menu item: {id:?}"),
            },
            UserEvent::StatsFetched { username, result } => self.on_stats_fetched(username, result),
            UserEvent::AvatarFetched { username, avatar } => {
                if let Some(window) = &mut self.preferences_window {
                    window.on_avatar_fetched(username, avatar);
                }
            }
            UserEvent::UpdateChecked(result) => self.on_update_checked(result),
            UserEvent::AccessKit(event) => {
                if let Some(window) =
                    self.preferences_window.as_mut().filter(|w| w.window_id() == event.window_id)
                {
                    window.on_accesskit_event(event.window_event);
                }
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(window) = self.preferences_window.as_mut().filter(|w| w.window_id() == window_id) else {
            return;
        };
        match window.on_window_event(&event) {
            WindowAction::None => {}
            WindowAction::Close => self.close_preferences(),
            WindowAction::Save { preferences, avatar } => {
                self.close_preferences();
                self.save_preferences(preferences, avatar);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut wake_at = self.next_tick;
        if let Some(window) = &mut self.preferences_window
            && let Some(deadline) = window.repaint_deadline()
        {
            if deadline <= Instant::now() {
                window.request_redraw();
            } else {
                wake_at = wake_at.min(deadline);
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(wake_at));
    }
}
