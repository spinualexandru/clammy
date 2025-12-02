use iced::{Color, Theme};
use std::sync::RwLock;

use crate::config::{parse_hex_color, parse_hex_color_with_alpha, Config};

// Global theme for component access
static GLOBAL_THEME: RwLock<Option<AppTheme>> = RwLock::new(None);

/// Update the global theme (called when config reloads)
pub fn set_global_theme(theme: &AppTheme) {
    if let Ok(mut guard) = GLOBAL_THEME.write() {
        *guard = Some(theme.clone());
    }
}

/// Get a copy of the current global theme
pub fn get_theme() -> AppTheme {
    GLOBAL_THEME
        .read()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_default()
}

/// Cached theme with pre-parsed colors for performance.
/// Colors are parsed once on config load/reload instead of on every access.
#[derive(Clone, Debug)]
pub struct AppTheme {
    // Cached parsed colors
    accent: Color,
    accent2: Color,
    info: Color,
    surface: Color,
    border: Color,
    muted: Color,
    hover: Color,
    text: Color,
    success: Color,
    danger: Color,
    background: Color,

    // Non-color settings
    font_size: f32,
    tray_widget_spacing: f32,
    tray_widget_padding: f32,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::from_config(&Config::default())
    }
}

impl AppTheme {
    pub fn from_config(config: &Config) -> Self {
        let theme = &config.theme;
        Self {
            accent: parse_hex_color(&theme.accent),
            accent2: parse_hex_color(&theme.accent2),
            info: parse_hex_color(&theme.info),
            surface: parse_hex_color_with_alpha(&theme.surface, theme.surface_alpha),
            border: parse_hex_color(&theme.border),
            muted: parse_hex_color(&theme.muted),
            hover: parse_hex_color_with_alpha(&theme.hover, theme.hover_alpha),
            text: parse_hex_color(&theme.text),
            success: parse_hex_color(&theme.success),
            danger: parse_hex_color(&theme.danger),
            background: parse_hex_color_with_alpha(&theme.background, theme.background_alpha),
            font_size: theme.font_size,
            tray_widget_spacing: theme.tray_widget_spacing,
            tray_widget_padding: theme.tray_widget_padding,
        }
    }

    /// Update theme from new config (re-parses all colors)
    pub fn update(&mut self, config: &Config) {
        *self = Self::from_config(config);
    }

    /// Blue accent color
    pub fn accent(&self) -> Color {
        self.accent
    }

    /// Purple accent color
    pub fn accent2(&self) -> Color {
        self.accent2
    }

    /// Cyan info color
    pub fn info(&self) -> Color {
        self.info
    }

    /// Surface/card background with alpha
    pub fn surface(&self) -> Color {
        self.surface
    }

    /// Border color
    pub fn border(&self) -> Color {
        self.border
    }

    /// Muted/comment text
    pub fn muted(&self) -> Color {
        self.muted
    }

    /// Hover state background with alpha
    pub fn hover(&self) -> Color {
        self.hover
    }

    /// Foreground/text color
    pub fn text(&self) -> Color {
        self.text
    }

    /// Success/green color
    pub fn success(&self) -> Color {
        self.success
    }

    /// Danger/red color
    pub fn danger(&self) -> Color {
        self.danger
    }

    /// Background color with alpha
    pub fn background(&self) -> Color {
        self.background
    }

    /// Font size in pixels
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Spacing between tray widgets in pixels
    pub fn tray_widget_spacing(&self) -> f32 {
        self.tray_widget_spacing
    }

    /// Horizontal padding inside each tray widget in pixels
    pub fn tray_widget_padding(&self) -> f32 {
        self.tray_widget_padding
    }
}

impl From<&AppTheme> for Theme {
    fn from(theme: &AppTheme) -> Self {
        Theme::custom(
            String::from("Clammy Theme"),
            iced::theme::Palette {
                background: Color::from_rgba(0.0, 0.0, 0.0, 0.0), // Transparent for bar
                text: theme.text,
                primary: theme.background,
                success: theme.success,
                danger: theme.danger,
            },
        )
    }
}
