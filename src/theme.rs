use iced::{Color, Theme};
use std::sync::RwLock;

use crate::config::{parse_hex_color, parse_hex_color_with_alpha, Config, ThemeConfig};

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

#[derive(Clone, Debug)]
pub struct AppTheme {
    config: ThemeConfig,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            config: ThemeConfig::default(),
        }
    }
}

impl AppTheme {
    pub fn from_config(config: &Config) -> Self {
        Self {
            config: config.theme.clone(),
        }
    }

    /// Update theme from new config
    pub fn update(&mut self, config: &Config) {
        self.config = config.theme.clone();
    }

    /// Blue accent color
    pub fn accent(&self) -> Color {
        parse_hex_color(&self.config.accent)
    }

    /// Purple accent color
    pub fn accent2(&self) -> Color {
        parse_hex_color(&self.config.accent2)
    }

    /// Cyan info color
    pub fn info(&self) -> Color {
        parse_hex_color(&self.config.info)
    }

    /// Surface/card background with alpha
    pub fn surface(&self) -> Color {
        parse_hex_color_with_alpha(&self.config.surface, self.config.surface_alpha)
    }

    /// Border color
    pub fn border(&self) -> Color {
        parse_hex_color(&self.config.border)
    }

    /// Muted/comment text
    pub fn muted(&self) -> Color {
        parse_hex_color(&self.config.muted)
    }

    /// Hover state background with alpha
    pub fn hover(&self) -> Color {
        parse_hex_color_with_alpha(&self.config.hover, self.config.hover_alpha)
    }

    /// Foreground/text color
    pub fn text(&self) -> Color {
        parse_hex_color(&self.config.text)
    }

    /// Success/green color
    pub fn success(&self) -> Color {
        parse_hex_color(&self.config.success)
    }

    /// Danger/red color
    pub fn danger(&self) -> Color {
        parse_hex_color(&self.config.danger)
    }

    /// Background color with alpha
    pub fn background(&self) -> Color {
        parse_hex_color_with_alpha(&self.config.background, self.config.background_alpha)
    }
}

impl From<&AppTheme> for Theme {
    fn from(theme: &AppTheme) -> Self {
        Theme::custom(
            String::from("Clammy Theme"),
            iced::theme::Palette {
                background: Color::from_rgba(0.0, 0.0, 0.0, 0.0), // Transparent for bar
                text: parse_hex_color(&theme.config.text),
                primary: parse_hex_color_with_alpha(
                    &theme.config.background,
                    theme.config.background_alpha,
                ),
                success: parse_hex_color(&theme.config.success),
                danger: parse_hex_color(&theme.config.danger),
            },
        )
    }
}
