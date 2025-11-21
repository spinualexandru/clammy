use iced::{Color, Theme};

#[derive(Clone, Copy, Debug)]
pub enum AppTheme {
    Dark,
    Light,
    Custom,
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme::Custom
    }
}

impl From<AppTheme> for Theme {
    fn from(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark => Theme::Dark,
            AppTheme::Light => Theme::Light,
            AppTheme::Custom => Theme::custom(
                String::from("Custom"),
                iced::theme::Palette {
                    background: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
                    text: Color::from_rgb(0.0, 0.0, 0.0),
                    primary: Color::from_rgb(0.0, 0.0, 0.0),
                    success: Color::from_rgb(0.0, 0.0, 0.0),
                    danger: Color::from_rgb(0.0, 0.0, 0.0),
                },
            ),
        }
    }
}

