use iced::widget::text;
use iced::{Element, Subscription};

use crate::hyprland_events::HyprlandSubscription;
use crate::theme::get_theme;

#[derive(Debug, Clone)]
pub struct WindowTitle {
    title: Option<String>,
    class: Option<String>,
    display_text: String,  // Cached display string
}

#[derive(Debug, Clone)]
pub enum Message {
    ActiveWindowChanged(Option<String>, Option<String>), // (title, class)
}

impl Default for WindowTitle {
    fn default() -> Self {
        Self {
            title: None,
            class: None,
            display_text: String::new(),
        }
    }
}

impl WindowTitle {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ActiveWindowChanged(title, class) => {
                self.title = title;
                self.class = class;

                // Update cached display text
                self.display_text.clear();
                if let (Some(t), Some(c)) = (&self.title, &self.class) {
                    use std::fmt::Write;
                    let _ = write!(&mut self.display_text, "{} - {}", c, t);
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let font_size = get_theme().font_size();
        text(&self.display_text)
            .size(font_size)
            .style(|theme: &iced::Theme| {
                text::Style {
                    color: Some(theme.palette().text),
                }
            })
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        HyprlandSubscription::new("hyprland-window-title-events")
            .on_active_window(|data| {
                let (title, class) = data.map(|(t, c)| (Some(t), Some(c))).unwrap_or((None, None));
                Message::ActiveWindowChanged(title, class)
            })
            .build()
    }
}
