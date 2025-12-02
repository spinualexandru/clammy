//! Shared tray widget helpers for consistent styling across components.

use iced::widget::{container, text};
use iced::{Element, Length};

use crate::theme::get_theme;

/// Creates a styled text widget for use in the tray area (right section).
/// Applies consistent font size, text color, padding, and vertical centering.
pub fn tray_text<'a, M: 'a>(content: &'a str) -> Element<'a, M> {
    let theme = get_theme();
    let text_widget = text(content)
        .size(theme.font_size())
        .style(|theme: &iced::Theme| iced::widget::text::Style {
            color: Some(theme.palette().text),
        });

    container(text_widget)
        .center_y(Length::Fill)
        .padding([0.0, theme.tray_widget_padding()])
        .into()
}
