//! The preferences window: a winit window rendered with egui via wgpu.
//!
//! The window and its GPU resources exist only while it is open, so the idle tray app holds no
//! GPU device.

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{
    Align, Align2, Atom, Color32, CornerRadius, FontId, Id, Key, Layout, Margin, Modifiers, Rect, Response,
    RichText, Sense, Stroke, StrokeKind, TextEdit, TextStyle, TextureHandle, TextureOptions, Theme,
    ThemePreference, Ui, Vec2, ViewportId, vec2,
};
use egui_wgpu::winit::Painter;
use egui_wgpu::{RendererOptions, WgpuConfiguration};
use egui_winit::accesskit_winit;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::WindowEvent;
#[cfg(target_os = "macos")]
use winit::event::{ElementState, MouseButton};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowButtons};

use crate::app::UserEvent;
use crate::command::EditAction;
use crate::preferences::{IconTheme, Preferences};

const TITLE: &str = "Streaker Preferences";
const SIZE: LogicalSize<f64> = LogicalSize::new(300.0, 516.0);
/// Height of the custom title bar drawn where the hidden native one was, matching it.
#[cfg(target_os = "macos")]
const TITLE_BAR_HEIGHT: f32 = 28.0;
/// Vertical center of the title, a little below the middle of the title bar so it isn't
/// crowded against the window's top edge.
#[cfg(target_os = "macos")]
const TITLE_CENTER_Y: f32 = 18.0;
/// Height of the strip that drags the window: the title bar plus the form's empty top margin,
/// reaching down to the first section title.
#[cfg(target_os = "macos")]
const DRAG_AREA_HEIGHT: f32 = TITLE_BAR_HEIGHT + 18.0;

/// What the window wants the app to do after handling an event.
#[derive(Debug)]
pub enum WindowAction {
    None,
    Close,
    /// Save the preferences, and cache their username's avatar if it loaded.
    Save {
        preferences: Preferences,
        avatar: Option<egui::ColorImage>,
    },
}

pub struct PreferencesWindow {
    // Field order matters: the painter's surface must be dropped before the window.
    painter: Painter,
    egui: egui_winit::State,
    window: Arc<Window>,
    form: PreferencesForm,
    next_repaint: Option<Instant>,
    /// Last known cursor height in physical pixels, to start window drags from the title bar.
    #[cfg(target_os = "macos")]
    cursor_y: f64,
    #[cfg(windows)]
    _menu: tray_icon::menu::Menu,
}

impl PreferencesWindow {
    pub fn open(
        event_loop: &ActiveEventLoop,
        preferences: &Preferences,
        avatar: Option<&egui::ColorImage>,
        proxy: EventLoopProxy<UserEvent>,
    ) -> anyhow::Result<Self> {
        let attributes = Window::default_attributes()
            .with_title(TITLE)
            .with_inner_size(SIZE)
            .with_resizable(false)
            .with_enabled_buttons(WindowButtons::CLOSE)
            .with_visible(false);
        // Hide the native title bar and its traffic lights; the content view extends
        // underneath and the form draws its own title bar there.
        #[cfg(target_os = "macos")]
        let attributes = {
            use winit::platform::macos::WindowAttributesExtMacOS;
            attributes
                .with_inner_size(LogicalSize::new(SIZE.width, SIZE.height + f64::from(TITLE_BAR_HEIGHT)))
                .with_titlebar_transparent(true)
                .with_title_hidden(true)
                .with_titlebar_buttons_hidden(true)
                .with_fullsize_content_view(true)
        };
        let window = Arc::new(event_loop.create_window(attributes)?);
        center_on_monitor(&window, event_loop);

        #[cfg(windows)]
        let _menu = crate::app_menu::attach_windows_menu(&window)?;

        let ctx = egui::Context::default();
        add_system_font(&ctx);
        ctx.options_mut(|options| {
            options.theme_preference = ThemePreference::System;
            options.zoom_with_keyboard = false;
        });
        for theme in [Theme::Light, Theme::Dark] {
            ctx.style_mut_of(theme, |style| apply_style(style, &Palette::of(theme)));
        }
        let mut form = PreferencesForm::new(preferences.clone(), proxy.clone(), &ctx);
        if let (Some(key), Some(image)) = (form.avatar_key(), avatar) {
            form.insert_avatar(&ctx, key, Some(image.clone()));
        }

        let mut painter = pollster::block_on(Painter::new(
            ctx.clone(),
            WgpuConfiguration::default(),
            false,
            RendererOptions::default(),
        ));
        pollster::block_on(painter.set_window(ViewportId::ROOT, Some(window.clone())))?;

        let mut egui = egui_winit::State::new(
            ctx,
            ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            window.theme(),
            painter.max_texture_side(),
        );
        // Must happen while the window is still hidden.
        egui.init_accesskit(event_loop, &window, proxy);

        let mut this = Self {
            painter,
            egui,
            window,
            form,
            next_repaint: None,
            #[cfg(target_os = "macos")]
            cursor_y: f64::MAX,
            #[cfg(windows)]
            _menu,
        };
        // Paint before showing so the window doesn't flash an empty frame.
        this.redraw();
        this.window.set_visible(true);
        this.focus();
        Ok(this)
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    pub fn focus(&self) {
        self.window.set_minimized(false);
        self.window.focus_window();
        self.window.request_redraw();
    }

    /// When egui next wants to repaint (e.g. for an animation or blinking cursor).
    pub fn repaint_deadline(&self) -> Option<Instant> {
        self.next_repaint
    }

    pub fn request_redraw(&mut self) {
        self.next_repaint = None;
        self.window.request_redraw();
    }

    pub fn on_window_event(&mut self, event: &WindowEvent) -> WindowAction {
        match event {
            WindowEvent::CloseRequested => return WindowAction::Close,
            WindowEvent::RedrawRequested => return self.redraw(),
            WindowEvent::Resized(size) => {
                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    self.painter.on_window_resized(ViewportId::ROOT, width, height);
                }
            }
            #[cfg(target_os = "macos")]
            WindowEvent::CursorMoved { position, .. } => self.cursor_y = position.y,
            #[cfg(target_os = "macos")]
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. }
                if self.cursor_y < f64::from(DRAG_AREA_HEIGHT) * self.window.scale_factor() =>
            {
                // Must be called while handling the mouse-down event.
                let _ = self.window.drag_window();
                return WindowAction::None;
            }
            _ => {}
        }

        if self.egui.on_window_event(&self.window, event).repaint {
            self.request_redraw();
        }
        WindowAction::None
    }

    pub fn on_avatar_fetched(&mut self, username: String, avatar: Option<egui::ColorImage>) {
        self.form.insert_avatar(self.egui.egui_ctx(), username, avatar);
        self.request_redraw();
    }

    pub fn on_accesskit_event(&mut self, event: accesskit_winit::WindowEvent) {
        match event {
            accesskit_winit::WindowEvent::InitialTreeRequested => self.egui.egui_ctx().enable_accesskit(),
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                self.egui.on_accesskit_action_request(request);
            }
            accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                self.egui.egui_ctx().disable_accesskit();
            }
        }
        self.request_redraw();
    }

    /// Feeds an Edit menu action to egui as the input it would have received from the keyboard.
    pub fn perform_edit(&mut self, action: EditAction) {
        let key = |key, modifiers| {
            [true, false].map(|pressed| egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            })
        };
        let events: Vec<egui::Event> = match action {
            EditAction::Undo => key(Key::Z, Modifiers::COMMAND).into(),
            EditAction::Redo => key(Key::Z, Modifiers::COMMAND | Modifiers::SHIFT).into(),
            EditAction::SelectAll => key(Key::A, Modifiers::COMMAND).into(),
            EditAction::Cut => vec![egui::Event::Cut],
            EditAction::Copy => vec![egui::Event::Copy],
            EditAction::Paste => match self.egui.clipboard_text() {
                Some(text) => vec![egui::Event::Paste(text)],
                None => return,
            },
        };
        self.egui.egui_input_mut().events.extend(events);
        self.request_redraw();
    }

    fn redraw(&mut self) -> WindowAction {
        self.next_repaint = None;
        let mut input = self.egui.take_egui_input(&self.window);
        let ctx = self.egui.egui_ctx().clone();

        // egui moves focus between controls with the arrow keys, but like a native app only Tab
        // should: the up and down arrows step the focused control's value instead. A focused text
        // field keeps them to move its cursor.
        self.form.arrow_steps = 0;
        if !ctx.text_edit_focused() {
            input.events.retain(|event| {
                let egui::Event::Key { key, pressed, modifiers, .. } = event else { return true };
                let step = match key {
                    Key::ArrowUp => 1,
                    Key::ArrowDown => -1,
                    Key::ArrowLeft | Key::ArrowRight => 0,
                    _ => return true,
                };
                if !modifiers.is_none() {
                    return true;
                }
                if *pressed {
                    self.form.arrow_steps += step;
                }
                false
            });
        }

        let mut action = WindowAction::None;
        let mut output = ctx.run_ui(input, |ui| {
            #[cfg(target_os = "macos")]
            title_bar(ui);
            action = match self.form.ui(ui) {
                Some(FormAction::Save) => WindowAction::Save {
                    preferences: self.form.preferences(),
                    avatar: self.form.loaded_avatar(),
                },
                Some(FormAction::Cancel) => WindowAction::Close,
                None => WindowAction::None,
            };
        });

        self.egui.handle_platform_output(&self.window, output.platform_output);
        let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
        let clear_color = ctx.global_style().visuals.panel_fill.to_normalized_gamma_f32();
        self.painter.paint_and_update_textures(
            ViewportId::ROOT,
            output.pixels_per_point,
            clear_color,
            &primitives,
            &mut output.textures_delta,
            Vec::new(),
            &self.window,
        );

        let repaint_delay = output
            .viewport_output
            .get(&ViewportId::ROOT)
            .map_or(Duration::MAX, |viewport| viewport.repaint_delay);
        if repaint_delay.is_zero() {
            self.window.request_redraw();
        } else if repaint_delay != Duration::MAX {
            self.next_repaint = Instant::now().checked_add(repaint_delay);
        }

        action
    }
}

/// The title, centered horizontally in the strip the hidden native title bar leaves at the top.
#[cfg(target_os = "macos")]
fn title_bar(ui: &mut Ui) {
    let top = ui.cursor().min;
    let center = egui::pos2(top.x + ui.available_width() / 2.0, top.y + TITLE_CENTER_Y);
    let color = Palette::for_ui(ui).weak_text;
    ui.painter().text(center, Align2::CENTER_CENTER, TITLE, FontId::proportional(13.0), color);
    ui.add_space(TITLE_BAR_HEIGHT);
}

fn center_on_monitor(window: &Window, event_loop: &ActiveEventLoop) {
    let Some(monitor) = window.current_monitor().or_else(|| event_loop.primary_monitor()) else {
        return;
    };
    let (monitor_pos, monitor_size, window_size) = (monitor.position(), monitor.size(), window.outer_size());
    let offset = |monitor: u32, window: u32| (i64::from(monitor) - i64::from(window)).max(0) as i32 / 2;
    window.set_outer_position(PhysicalPosition::new(
        monitor_pos.x + offset(monitor_size.width, window_size.width),
        monitor_pos.y + offset(monitor_size.height, window_size.height),
    ));
}

/// Colors for one theme: neutral greys with a blue accent.
#[derive(Clone, Copy)]
struct Palette {
    background: Color32,
    card: Color32,
    card_stroke: Color32,
    separator: Color32,
    text: Color32,
    weak_text: Color32,
    accent: Color32,
    toggle_off: Color32,
    error: Color32,
    warning: Color32,
}

impl Palette {
    fn of(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self {
                background: Color32::from_rgb(0xf3, 0xf4, 0xf6),
                card: Color32::WHITE,
                card_stroke: Color32::from_rgb(0xdf, 0xe2, 0xe6),
                separator: Color32::from_rgb(0xea, 0xec, 0xef),
                text: Color32::from_rgb(0x1f, 0x23, 0x28),
                weak_text: Color32::from_rgb(0x65, 0x6d, 0x76),
                accent: Color32::from_rgb(0x09, 0x69, 0xda),
                toggle_off: Color32::from_rgb(0xd0, 0xd7, 0xde),
                error: Color32::from_rgb(0xcf, 0x22, 0x2e),
                warning: Color32::from_rgb(0x9a, 0x67, 0x00),
            },
            Theme::Dark => Self {
                background: Color32::from_rgb(0x1c, 0x1c, 0x1e),
                card: Color32::from_rgb(0x2a, 0x2a, 0x2c),
                card_stroke: Color32::from_rgb(0x3a, 0x3a, 0x3c),
                separator: Color32::from_rgb(0x33, 0x33, 0x35),
                text: Color32::from_rgb(0xe8, 0xe8, 0xea),
                weak_text: Color32::from_rgb(0x98, 0x98, 0x9d),
                accent: Color32::from_rgb(0x0a, 0x84, 0xff),
                toggle_off: Color32::from_rgb(0x48, 0x48, 0x4a),
                error: Color32::from_rgb(0xf8, 0x51, 0x49),
                warning: Color32::from_rgb(0xd2, 0x99, 0x22),
            },
        }
    }

    fn for_ui(ui: &Ui) -> Self {
        Self::of(if ui.visuals().dark_mode { Theme::Dark } else { Theme::Light })
    }
}

/// Prefers the OS UI font over egui's bundled Ubuntu, which stays as the fallback for missing
/// glyphs or a missing font file. System fonts are read from disk as their licences forbid
/// bundling them.
fn add_system_font(ctx: &egui::Context) {
    #[cfg(target_os = "macos")]
    let path = Some(std::path::PathBuf::from("/System/Library/Fonts/SFNS.ttf"));
    #[cfg(windows)]
    let path = std::env::var_os("WINDIR").map(|dir| std::path::Path::new(&dir).join(r"Fonts\segoeui.ttf"));
    #[cfg(not(any(target_os = "macos", windows)))]
    let path: Option<std::path::PathBuf> = None;

    let Some(path) = path else { return };
    match std::fs::read(&path) {
        Ok(bytes) => ctx.add_font(egui::epaint::text::FontInsert::new(
            "system-ui",
            egui::FontData::from_owned(bytes),
            vec![egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            }],
        )),
        Err(err) => log::warn!("Failed to load system font {}: {err}", path.display()),
    }
}

fn apply_style(style: &mut egui::Style, palette: &Palette) {
    use egui::FontFamily::Proportional;
    style.text_styles = [
        (TextStyle::Heading, FontId::new(20.0, Proportional)),
        (TextStyle::Body, FontId::new(13.5, Proportional)),
        (TextStyle::Button, FontId::new(13.5, Proportional)),
        (TextStyle::Small, FontId::new(11.5, Proportional)),
        (TextStyle::Monospace, FontId::new(13.5, egui::FontFamily::Monospace)),
    ]
    .into();
    style.spacing.button_padding = vec2(12.0, 5.0);
    style.spacing.interact_size.y = 28.0;
    style.spacing.combo_height = 240.0;
    // Native settings windows don't let you select label text or show an I-beam over it.
    style.interaction.selectable_labels = false;

    let visuals = &mut style.visuals;
    visuals.panel_fill = palette.background;
    visuals.window_fill = palette.card;
    visuals.extreme_bg_color = palette.card;
    visuals.override_text_color = Some(palette.text);
    visuals.error_fg_color = palette.error;
    visuals.warn_fg_color = palette.warning;
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, palette.accent);
    visuals.text_cursor.stroke = Stroke::new(2.0, palette.accent);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0, palette.card_stroke);

    let widgets = &mut visuals.widgets;
    for widget in [&mut widgets.inactive, &mut widgets.hovered, &mut widgets.active, &mut widgets.open] {
        widget.corner_radius = CornerRadius::same(6);
        widget.expansion = 0.0;
    }
    widgets.inactive.weak_bg_fill = palette.card;
    widgets.inactive.bg_stroke = Stroke::new(1.0, palette.card_stroke);
    widgets.hovered.weak_bg_fill = palette.separator;
    widgets.hovered.bg_stroke = Stroke::new(1.0, palette.card_stroke);
    widgets.active.weak_bg_fill = palette.card_stroke;
    widgets.active.bg_stroke = Stroke::new(1.0, palette.card_stroke);
    widgets.open.weak_bg_fill = palette.separator;
    widgets.open.bg_stroke = Stroke::new(1.0, palette.accent);
}

enum FormAction {
    Save,
    Cancel,
}

/// How long the username must stay unchanged before its avatar is fetched.
const AVATAR_DEBOUNCE: Duration = Duration::from_millis(400);
const AVATAR_SIZE: f32 = 34.0;
/// How long each of a theme's icons shows before the next.
const ICON_CYCLE: Duration = Duration::from_millis(1000);

enum AvatarState {
    Loading,
    Loaded(TextureHandle, egui::ColorImage),
    /// There's no such user, or the fetch failed.
    Missing,
}

/// The editable copy of the preferences shown in the window.
struct PreferencesForm {
    draft: Preferences,
    focus_username: bool,
    proxy: EventLoopProxy<UserEvent>,
    /// Avatars by lowercase username, which GitHub treats case-insensitively.
    avatars: HashMap<String, AvatarState>,
    /// When the username was last edited; `None` fetches the avatar without waiting.
    username_edited_at: Option<Instant>,
    /// Each theme's icons, as returned by [`crate::icons::preview_images`].
    theme_icons: Vec<(IconTheme, [TextureHandle; 3])>,
    /// When the window opened, to cycle the theme icons from the first.
    opened_at: Instant,
    /// Whether focus is being moved with the keyboard, so Enter goes to the focused control.
    keyboard_focus: bool,
    /// Up arrow presses minus down arrow presses this frame, for the focused control.
    arrow_steps: i32,
}

impl PreferencesForm {
    fn new(preferences: Preferences, proxy: EventLoopProxy<UserEvent>, ctx: &egui::Context) -> Self {
        let theme_icons = IconTheme::ALL
            .into_iter()
            .map(|theme| {
                let textures = crate::icons::preview_images(theme).map(|encoded| {
                    let mut image = image::load_from_memory(encoded)
                        .expect("embedded icon is a valid image")
                        .into_rgba8();
                    // Make the silhouette white so it can be tinted to match the text color.
                    for pixel in image.pixels_mut() {
                        pixel.0[..3].fill(255);
                    }
                    let size = [image.width() as usize, image.height() as usize];
                    let image = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
                    ctx.load_texture(format!("icon-{}", theme.label()), image, TextureOptions::LINEAR)
                });
                (theme, textures)
            })
            .collect();
        Self {
            draft: preferences,
            focus_username: true,
            proxy,
            avatars: HashMap::new(),
            username_edited_at: None,
            theme_icons,
            opened_at: Instant::now(),
            keyboard_focus: false,
            arrow_steps: 0,
        }
    }

    fn preferences(&self) -> Preferences {
        Preferences { username: self.draft.username.trim().to_owned(), ..self.draft.clone() }
    }

    fn username_is_valid(&self) -> bool {
        let username = self.draft.username.trim();
        username.is_empty() || crate::contribution::is_valid_username(username)
    }

    /// The avatar key for the username being edited, if it's one that can have an avatar.
    fn avatar_key(&self) -> Option<String> {
        let username = self.draft.username.trim();
        crate::contribution::is_valid_username(username).then(|| username.to_ascii_lowercase())
    }

    /// Starts fetching the current username's avatar once typing has paused.
    fn fetch_avatar_when_idle(&mut self, ctx: &egui::Context) {
        let Some(key) = self.avatar_key() else { return };
        if self.avatars.contains_key(&key) {
            return;
        }
        let waited = self.username_edited_at.map_or(AVATAR_DEBOUNCE, |at| at.elapsed());
        if waited < AVATAR_DEBOUNCE {
            ctx.request_repaint_after(AVATAR_DEBOUNCE - waited);
            return;
        }

        self.avatars.insert(key.clone(), AvatarState::Loading);
        crate::avatar::fetch_in_background(key, self.proxy.clone());
    }

    fn insert_avatar(&mut self, ctx: &egui::Context, username: String, avatar: Option<egui::ColorImage>) {
        let state = match avatar {
            Some(image) => AvatarState::Loaded(
                ctx.load_texture(format!("avatar-{username}"), image.clone(), TextureOptions::LINEAR),
                image,
            ),
            None => AvatarState::Missing,
        };
        self.avatars.insert(username, state);
    }

    fn avatar(&self) -> Option<&AvatarState> {
        self.avatars.get(&self.avatar_key()?)
    }

    fn loaded_avatar(&self) -> Option<egui::ColorImage> {
        match self.avatar()? {
            AvatarState::Loaded(_, image) => Some(image.clone()),
            _ => None,
        }
    }

    fn ui(&mut self, ui: &mut Ui) -> Option<FormAction> {
        // Whether focus was last moved with the keyboard rather than by clicking.
        let (tabbed, pointer_pressed) =
            ui.input(|input| (input.key_pressed(Key::Tab), input.pointer.any_pressed()));
        if tabbed {
            self.keyboard_focus = true;
        } else if pointer_pressed {
            self.keyboard_focus = false;
        }

        // Enter saves, unless another control was focused with the keyboard: then it activates
        // that control.
        let username_id = Id::new("username");
        let enter_saves =
            !self.keyboard_focus || ui.memory(|memory| memory.focused().is_none_or(|id| id == username_id));
        // Escape closes an open dropdown before it closes the window.
        let escape_closes = !egui::Popup::is_any_open(ui.ctx());
        let (enter, escape, close) = ui.input_mut(|input| {
            (
                enter_saves && input.consume_key(Modifiers::NONE, Key::Enter),
                escape_closes && input.consume_key(Modifiers::NONE, Key::Escape),
                input.consume_key(Modifiers::COMMAND, Key::W),
            )
        });
        let mut action = match (enter && self.username_is_valid(), escape || close) {
            (true, _) => Some(FormAction::Save),
            (_, true) => Some(FormAction::Cancel),
            _ => None,
        };

        let palette = Palette::for_ui(ui);
        egui::Frame::central_panel(ui.style()).inner_margin(Margin::symmetric(20, 18)).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.spacing_mut().item_spacing = vec2(8.0, 6.0);

            section_title(ui, &palette, "GitHub Account");
            self.username_field(ui, &palette);
            ui.add_space(14.0);

            section_title(ui, &palette, "General");
            self.general_card(ui, &palette);
            ui.add_space(14.0);

            section_title(ui, &palette, "Menu Bar Icon");
            self.icon_theme_picker(ui, &palette);

            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                // Cancel is added before Save so Tab reaches it first.
                let button_size = vec2(92.0, 30.0);
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), button_size.y),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        let cancel_badge = Id::new("cancel-badge");
                        let cancel =
                            egui::Button::new(("Cancel", Atom::custom(cancel_badge, KEY_BADGE_SIZE)))
                                .gap(KEY_BADGE_GAP)
                                .min_size(vec2(button_size.x + 12.0, button_size.y))
                                .atom_ui(ui);
                        if let Some(rect) = cancel.rect(cancel_badge) {
                            key_badge(ui, rect, palette.weak_text, KeyBadge::Escape);
                        }
                        let cancel = cancel.response.on_hover_cursor(egui::CursorIcon::PointingHand);
                        focus_ring(ui, &cancel, palette.accent, 6);
                        if cancel.clicked() {
                            action = Some(FormAction::Cancel);
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let save_badge = Id::new("save-badge");
                            let save = egui::Button::new((
                                RichText::new("Save").strong().color(Color32::WHITE),
                                Atom::custom(save_badge, ENTER_BADGE_SIZE),
                            ))
                            .fill(palette.accent)
                            .stroke(Stroke::NONE)
                            .gap(KEY_BADGE_GAP)
                            .min_size(vec2(button_size.x - 8.0, button_size.y));
                            let save =
                                ui.add_enabled_ui(self.username_is_valid(), |ui| save.atom_ui(ui)).inner;
                            if let Some(rect) = save.rect(save_badge) {
                                key_badge(ui, rect, Color32::WHITE, KeyBadge::Enter);
                            }
                            let save = save.response.on_hover_cursor(egui::CursorIcon::PointingHand);
                            focus_ring(ui, &save, palette.accent, 6);
                            if save.clicked() {
                                action = Some(FormAction::Save);
                            }
                        });
                    },
                );
            });
        });

        action
    }

    fn username_field(&mut self, ui: &mut Ui, palette: &Palette) {
        self.fetch_avatar_when_idle(ui.ctx());

        ui.horizontal(|ui| {
            self.avatar_ui(ui, palette);

            let valid = self.username_is_valid();
            let visuals = ui.visuals_mut();
            visuals.selection.stroke.width *= 2.0;
            for widget in [&mut visuals.widgets.inactive, &mut visuals.widgets.hovered] {
                widget.bg_stroke.width *= 2.0;
            }
            for widget in
                [&mut visuals.widgets.inactive, &mut visuals.widgets.hovered, &mut visuals.widgets.active]
            {
                widget.corner_radius = CornerRadius::same(10);
            }
            if !valid {
                visuals.selection.stroke.color = palette.error;
                for widget in [&mut visuals.widgets.inactive, &mut visuals.widgets.hovered] {
                    widget.bg_stroke.color = palette.error;
                }
            }
            let edit = ui.add(
                TextEdit::singleline(&mut self.draft.username)
                    .id(Id::new("username"))
                    .hint_text("octocat")
                    .margin(Margin::symmetric(12, 4))
                    .min_size(vec2(0.0, AVATAR_SIZE))
                    .vertical_align(Align::Center)
                    .desired_width(f32::INFINITY),
            );
            edit.widget_info(|| {
                let label = if valid { "GitHub username" } else { "GitHub username, not valid" };
                egui::WidgetInfo::text_edit(true, "", &self.draft.username, label)
            });
            if edit.changed() {
                self.username_edited_at = Some(Instant::now());
            }
            if std::mem::take(&mut self.focus_username) {
                edit.request_focus();
            }
        });
    }

    /// The avatar beside the username field, or a placeholder until it loads.
    fn avatar_ui(&self, ui: &mut Ui, palette: &Palette) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(AVATAR_SIZE), Sense::hover());
        let corner_radius = CornerRadius::same(10);
        let painter = ui.painter();
        if let Some(AvatarState::Loaded(texture, _)) = self.avatar() {
            egui::Image::new(egui::load::SizedTexture::from_handle(texture))
                .corner_radius(corner_radius)
                .paint_at(ui, rect);
            painter.rect_stroke(
                rect,
                corner_radius,
                Stroke::new(1.0, palette.card_stroke),
                StrokeKind::Inside,
            );
            return;
        }

        painter.rect_filled(rect, corner_radius, palette.separator);
        // A generic person silhouette.
        let (center, half) = (rect.center(), AVATAR_SIZE / 2.0);
        let color = palette.toggle_off;
        painter.circle_filled(center - vec2(0.0, half * 0.22), half * 0.32, color);
        painter.with_clip_rect(rect.shrink(half * 0.2)).circle_filled(
            center + vec2(0.0, half * 0.95),
            half * 0.62,
            color,
        );
    }

    fn general_card(&mut self, ui: &mut Ui, palette: &Palette) {
        card(palette).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            setting_row(ui, palette, "Launch at login", None, |ui| {
                toggle(ui, palette, &mut self.draft.launch_at_login, "Launch at login");
            });
            row_separator(ui, palette);
            setting_row(ui, palette, "Streak reminder", Some("Notify me if my streak is at risk"), |ui| {
                toggle(ui, palette, &mut self.draft.reminder_enabled, "Streak reminder");
            });
            row_separator(ui, palette);
            ui.add_enabled_ui(self.draft.reminder_enabled, |ui| {
                setting_row(ui, palette, "Reminder time", None, |ui| {
                    // Laid out left to right so Tab reaches the hour before the minute.
                    let colon = RichText::new(":").strong();
                    let colon_width = egui::WidgetText::from(colon.clone())
                        .into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, TextStyle::Body)
                        .size()
                        .x;
                    let width = number_picker_width(ui, 0..24)
                        + number_picker_width(ui, 0..60)
                        + colon_width
                        + ui.spacing().item_spacing.x * 2.0;
                    let size = vec2(width, ui.spacing().interact_size.y);
                    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                        let time = &mut self.draft.reminder_time;
                        let steps = self.arrow_steps;
                        number_picker(
                            ui,
                            palette,
                            "reminder-hour",
                            "Reminder hour",
                            &mut time.hour,
                            0..24,
                            steps,
                        );
                        ui.label(colon);
                        number_picker(
                            ui,
                            palette,
                            "reminder-minute",
                            "Reminder minute",
                            &mut time.minute,
                            0..60,
                            steps,
                        );
                    });
                });
            });
        });
    }

    fn icon_theme_picker(&mut self, ui: &mut Ui, palette: &Palette) {
        let elapsed = self.opened_at.elapsed();
        let icon_index = (elapsed.as_millis() / ICON_CYCLE.as_millis()) as usize % 3;
        let until_next = ICON_CYCLE.as_millis() - elapsed.as_millis() % ICON_CYCLE.as_millis();
        ui.ctx().request_repaint_after(Duration::from_millis(until_next as u64));
        card(palette).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (i, (theme, textures)) in self.theme_icons.iter().enumerate() {
                if i > 0 {
                    row_separator(ui, palette);
                }
                let selected = self.draft.icon_theme == *theme;
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
                if response.clicked() {
                    self.draft.icon_theme = *theme;
                }
                focus_on_click(&response);
                response.widget_info(|| {
                    egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, selected, theme.label())
                });
                let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                if !ui.is_rect_visible(rect) {
                    continue;
                }

                // The theme's icon, cycling through its variants together with the other rows.
                let icon_size = 18.0;
                let icon_rect = Rect::from_center_size(
                    rect.left_center() + vec2(icon_size / 2.0, 0.0),
                    Vec2::splat(icon_size),
                );
                egui::Image::new(egui::load::SizedTexture::from_handle(&textures[icon_index]))
                    .tint(palette.text)
                    .paint_at(ui, icon_rect);

                let painter = ui.painter();
                painter.text(
                    rect.left_center() + vec2(icon_size + 10.0, 0.0),
                    Align2::LEFT_CENTER,
                    theme.label(),
                    TextStyle::Body.resolve(ui.style()),
                    palette.text,
                );

                // A radio button on the right, filled with the accent when selected.
                let radius = 9.0;
                let radio_center = rect.right_center() - vec2(radius, 0.0);
                let t = ui.ctx().animate_bool_responsive(response.id, selected);
                painter.circle_stroke(radio_center, radius - 0.75, Stroke::new(1.5, palette.toggle_off));
                painter.circle_filled(radio_center, radius, palette.accent.gamma_multiply(t));
                painter.circle_filled(radio_center, 3.5 * t, Color32::WHITE);
                if response.has_focus() {
                    painter.circle_stroke(
                        radio_center,
                        radius + 3.0,
                        Stroke::new(2.0, palette.accent.gamma_multiply(0.6)),
                    );
                }
            }
        });
    }
}

/// A rounded, outlined container for a group of settings.
fn card(palette: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.card)
        .stroke(Stroke::new(1.0, palette.card_stroke))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(14, 2))
}

fn section_title(ui: &mut Ui, palette: &Palette, text: &str) {
    ui.label(RichText::new(text.to_uppercase()).small().strong().color(palette.weak_text));
    ui.add_space(2.0);
}

/// A row with a title (and optional description) on the left and a control on the right.
fn setting_row(
    ui: &mut Ui,
    palette: &Palette,
    title: &str,
    description: Option<&str>,
    control: impl FnOnce(&mut Ui),
) {
    let height = if description.is_some() { 54.0 } else { 44.0 };
    // Laid out from the right so the control keeps its size and the text takes what's left.
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), height),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.set_min_height(height);
            control(ui);
            // Sized to the text, so the row's centered layout centers it vertically.
            let spacing = 2.0;
            let mut text_height = ui.text_style_height(&TextStyle::Body);
            if description.is_some() {
                text_height += spacing + ui.text_style_height(&TextStyle::Small);
            }
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), text_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.y = spacing;
                    ui.label(title);
                    if let Some(description) = description {
                        ui.label(RichText::new(description).small().color(palette.weak_text));
                    }
                },
            );
        },
    );
}

fn row_separator(ui: &mut Ui, palette: &Palette) {
    let rect = ui.available_rect_before_wrap();
    ui.painter().hline(rect.x_range(), rect.top(), Stroke::new(1.0, palette.separator));
}

/// An iOS-style switch.
fn toggle(ui: &mut Ui, palette: &Palette, on: &mut bool, label: &str) -> Response {
    let size = vec2(38.0, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    focus_on_click(&response);
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, label));
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    focus_ring(ui, &response, palette.accent, (size.y / 2.0) as u8);

    if ui.is_rect_visible(rect) {
        let t = ui.ctx().animate_bool_responsive(response.id, *on);
        let radius = rect.height() / 2.0;
        let painter = ui.painter();
        painter.rect_filled(rect, radius, palette.toggle_off.lerp_to_gamma(palette.accent, t));
        let knob =
            egui::pos2(egui::lerp((rect.left() + radius)..=(rect.right() - radius), t), rect.center().y);
        painter.circle_filled(knob + vec2(0.0, 0.5), radius - 2.0, Color32::from_black_alpha(40));
        painter.circle_filled(knob, radius - 2.5, Color32::WHITE);
    }
    response
}

/// The width that fits the widest value in `range`. The system font's digits aren't all the same
/// width, so sizing to the selected value would shift the layout as the arrow keys step it.
fn number_picker_width(ui: &Ui, range: std::ops::Range<u8>) -> f32 {
    let widest_text = range
        .map(|n| {
            egui::WidgetText::from(format!("{n:02}"))
                .into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, TextStyle::Button)
                .size()
                .x
        })
        .fold(0.0, f32::max);
    widest_text + ui.spacing().button_padding.x * 2.0
}

const POPUP_EXTRA_WIDTH: f32 = 16.0;

/// A button showing a zero-padded number that opens a list of `range` to pick from. Drawn by hand
/// rather than with a `ComboBox` to centre the number, drop the arrow and open the list leftwards.
fn number_picker(
    ui: &mut Ui,
    palette: &Palette,
    id: &str,
    label: &str,
    value: &mut u8,
    range: std::ops::Range<u8>,
    arrow_steps: i32,
) {
    let previous = *value;
    let size = vec2(number_picker_width(ui, range.clone()), ui.spacing().interact_size.y);
    let (_, rect) = ui.allocate_space(size);
    let response = ui.interact(rect, ui.make_persistent_id(id), Sense::click());

    let is_open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&response));
    // Set while the list is closed, so it opens scrolled to the selected number.
    let scroll_pending_id = response.id.with("scroll_pending");
    if !is_open {
        ui.data_mut(|data| data.insert_temp(scroll_pending_id, true));
    }
    if ui.is_rect_visible(rect) {
        let visuals = if is_open { &ui.visuals().widgets.open } else { ui.style().interact(&response) };
        ui.painter().rect(
            rect.expand(visuals.expansion),
            visuals.corner_radius,
            visuals.weak_bg_fill,
            visuals.bg_stroke,
            StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("{value:02}"),
            TextStyle::Button.resolve(ui.style()),
            visuals.text_color(),
        );
    }

    // Wider than the button to leave the numbers room beside the scroll bar, and anchored to the
    // right edge so the list always extends leftwards.
    egui::Popup::menu(&response)
        .align(egui::RectAlign::BOTTOM_END)
        .align_alternatives(&[egui::RectAlign::TOP_END])
        .width(rect.width() + POPUP_EXTRA_WIDTH)
        .show(|ui| {
            ui.set_min_width(ui.available_width());
            egui::ScrollArea::vertical().max_height(ui.spacing().combo_height).show(ui, |ui| {
                // No gaps between options: the popup closes on any click, so a click in a gap would
                // close it without picking anything.
                ui.spacing_mut().item_spacing.y = 0.0;
                // The first time a popup shows, egui lays it out invisibly to size it, so wait for
                // the pass that's drawn before scrolling.
                let scroll_pending = !ui.is_sizing_pass()
                    && ui.data_mut(|data| data.remove_temp::<bool>(scroll_pending_id).unwrap_or(false));
                for n in range.clone().rev() {
                    let option = ui.selectable_value(value, n, format!("{n:02}"));
                    if scroll_pending && n == *value {
                        option.scroll_to_me_animation(
                            Some(Align::Center),
                            egui::style::ScrollAnimation::none(),
                        );
                    }
                }
            });
        });
    focus_on_click(&response);
    // The arrow keys step the value while focused, wrapping around like a native time picker.
    if response.has_focus() && arrow_steps != 0 {
        let len = i32::from(range.end - range.start);
        let offset = (i32::from(*value - range.start) + arrow_steps).rem_euclid(len);
        *value = range.start + offset as u8;
        ui.ctx().request_repaint();
    }
    // Picking an option is a click outside the combo box, which takes its focus away.
    if *value != previous {
        response.request_focus();
    }
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ui.is_enabled(), format!("{label} {value:02}"))
    });
    focus_ring(ui, &response, palette.accent, 6);
}

/// Focuses a control when it's clicked, like a native app, so Tab continues from it. egui only
/// does this for text edits.
fn focus_on_click(response: &Response) {
    if response.clicked() {
        response.request_focus();
    }
}

/// Outlines a control that has focus. Painted outside it, so the layout doesn't move.
const KEY_BADGE_SIZE: Vec2 = vec2(26.0, 17.0);
/// The enter badge holds an icon rather than text, so it needs less width.
const ENTER_BADGE_SIZE: Vec2 = vec2(20.0, KEY_BADGE_SIZE.y);
/// The space between a button's label and its key badge.
const KEY_BADGE_GAP: f32 = 10.0;

/// The keyboard shortcut a button's badge shows.
enum KeyBadge {
    Escape,
    Enter,
}

/// Paints a small keycap in `rect` showing the button's keyboard shortcut.
fn key_badge(ui: &Ui, rect: Rect, color: Color32, key: KeyBadge) {
    let painter = ui.painter();
    let color = if ui.is_enabled() { color } else { color.gamma_multiply(0.5) };
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0, color.gamma_multiply(0.6)),
        StrokeKind::Inside,
    );
    match key {
        KeyBadge::Escape => {
            painter.text(rect.center(), Align2::CENTER_CENTER, "ESC", FontId::proportional(10.0), color);
        }
        KeyBadge::Enter => {
            // A return arrow: down from the top right, then left to an arrowhead.
            let stroke = Stroke::new(1.2, color);
            let (center, w, h, head) = (rect.center(), 3.0, 1.5, 1.8);
            let corner = center + vec2(w, h);
            let tip = center + vec2(-w, h);
            painter.line_segment([center + vec2(w, -h - 1.0), corner], stroke);
            painter.line_segment([corner, tip], stroke);
            painter.line_segment([tip, tip + vec2(head, -head)], stroke);
            painter.line_segment([tip, tip + vec2(head, head)], stroke);
        }
    }
}

fn focus_ring(ui: &Ui, response: &Response, accent: Color32, corner_radius: u8) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.expand(2.0),
            CornerRadius::same(corner_radius + 2),
            Stroke::new(2.0, accent.gamma_multiply(0.6)),
            StrokeKind::Outside,
        );
    }
}
