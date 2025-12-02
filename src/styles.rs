//! Shared styling functions for consistent UI appearance.

use iced::widget::button;
use iced::{Border, Color};

/// Creates a button style function for interactive elements with hover states.
///
/// # Arguments
/// * `is_active` - Whether the button is in an active/selected state
/// * `enabled` - Whether the button is enabled (clickable)
/// * `text_color` - Color for enabled text
/// * `muted_color` - Color for inactive/disabled text
/// * `hover_bg` - Background color on hover
///
/// # Returns
/// A closure suitable for `button.style()`
pub fn interactive_button_style(
    is_active: bool,
    enabled: bool,
    text_color: Color,
    muted_color: Color,
    hover_bg: Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (background, txt) = if is_active {
            (None, text_color)
        } else if !enabled {
            (None, muted_color)
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => {
                    (Some(hover_bg.into()), text_color)
                }
                _ => (None, muted_color),
            }
        };

        button::Style {
            background,
            text_color: txt,
            border: Border::default(),
            shadow: Default::default(),
        }
    }
}

/// Creates a button style for menu items with optional active state highlight.
///
/// # Arguments
/// * `is_active` - Whether this menu item is currently active/open
/// * `enabled` - Whether the menu item is enabled
/// * `text_color` - Color for enabled text
/// * `muted_color` - Color for disabled text
/// * `hover_bg` - Background color on hover
/// * `active_bg` - Background color when active (optional, uses hover_bg * 1.5 alpha if None)
pub fn menu_button_style(
    is_active: bool,
    enabled: bool,
    text_color: Color,
    muted_color: Color,
    hover_bg: Color,
    active_bg: Option<Color>,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    let active_bg = active_bg.unwrap_or_else(|| {
        Color::from_rgba(hover_bg.r, hover_bg.g, hover_bg.b, (hover_bg.a * 1.5).min(1.0))
    });

    move |_theme, status| {
        let bg = if !enabled {
            None
        } else if is_active {
            Some(active_bg.into())
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => Some(hover_bg.into()),
                _ => None,
            }
        };

        button::Style {
            background: bg,
            text_color: if enabled { text_color } else { muted_color },
            border: Border::default(),
            shadow: Default::default(),
        }
    }
}
