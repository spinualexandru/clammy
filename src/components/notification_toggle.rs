use iced::widget::{button, text};
use iced::{Border, Element, Subscription, Task};
use std::process::Command;

use crate::theme::get_theme;

#[derive(Debug, Clone, Default)]
pub struct NotificationToggle;

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    Toggled,
}

impl NotificationToggle {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Toggle => Task::perform(Self::toggle_panel(), |_| Message::Toggled),
            Message::Toggled => Task::none(),
        }
    }

    async fn toggle_panel() {
        let _ = Command::new("swaync-client")
            .arg("--toggle-panel")
            .spawn();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme = get_theme();
        let hover_bg = theme.hover();
        let text_color = theme.text();
        let font_size = theme.font_size();

        // Nerd Font bell icon
        button(text("󰂚").size(font_size))
            .padding([0, 8])
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered => Some(hover_bg.into()),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    border: Border {
                        radius: 4.0.into(),
                        ..Border::default()
                    },
                    text_color,
                    shadow: Default::default(),
                }
            })
            .on_press(Message::Toggle)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
