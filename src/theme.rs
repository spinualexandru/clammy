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
                String::from("Tokyo Night"),
                iced::theme::Palette {
                    background: Color::from_rgba(0.0, 0.0, 0.0, 0.0), // Transparent for bar
                    text: Color::from_rgb8(0xc0, 0xca, 0xf5),          // #c0caf5 - foreground
                    primary: Color::from_rgba8(0x1a, 0x1b, 0x26, 0.85), // #1a1b26 - bg with alpha
                    success: Color::from_rgb8(0x9e, 0xce, 0x6a),       // #9ece6a - green
                    danger: Color::from_rgb8(0xf7, 0x76, 0x8e),        // #f7768e - red
                },
            ),
        }
    }
}

impl AppTheme {
    /// Blue accent color (#7aa2f7)
    pub fn accent(&self) -> Color {
        Color::from_rgb8(0x7a, 0xa2, 0xf7)
    }

    /// Purple accent color (#bb9af7)
    pub fn accent2(&self) -> Color {
        Color::from_rgb8(0xbb, 0x9a, 0xf7)
    }

    /// Cyan info color (#7dcfff)
    pub fn info(&self) -> Color {
        Color::from_rgb8(0x7d, 0xcf, 0xff)
    }

    /// Surface/card background (#24283b with alpha)
    pub fn surface(&self) -> Color {
        Color::from_rgba8(0x24, 0x28, 0x3b, 0.94)
    }

    /// Border color (#414868)
    pub fn border(&self) -> Color {
        Color::from_rgb8(0x41, 0x48, 0x68)
    }

    /// Muted/comment text (#565f89)
    pub fn muted(&self) -> Color {
        Color::from_rgb8(0x56, 0x5f, 0x89)
    }

    /// Hover state background (#414868 with alpha)
    pub fn hover(&self) -> Color {
        Color::from_rgba8(0x41, 0x48, 0x68, 0.5)
    }

    /// Foreground/text color (#c0caf5)
    pub fn text(&self) -> Color {
        Color::from_rgb8(0xc0, 0xca, 0xf5)
    }
}
