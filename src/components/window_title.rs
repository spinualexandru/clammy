use hyprland::event_listener::AsyncEventListener;
use iced::futures::SinkExt;
use iced::stream;
use iced::widget::text;
use iced::{Element, Subscription};
use std::future;

#[derive(Debug, Clone)]
pub struct WindowTitle {
    title: Option<String>,
    class: Option<String>,
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
        }
    }
}

impl WindowTitle {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ActiveWindowChanged(title, class) => {
                self.title = title;
                self.class = class;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if let (Some(title), Some(class)) = (&self.title, &self.class) {
            text(format!("{} - {}", class, title))
                .style(|theme: &iced::Theme| {
                    text::Style {
                        color: Some(theme.palette().text),
                    }
                })
                .into()
        } else {
            text("")
                .style(|theme: &iced::Theme| {
                    text::Style {
                        color: Some(theme.palette().text),
                    }
                })
                .into()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run_with_id(
            "hyprland-window-title-events",
            stream::channel(100, |output| async move {
                let mut listener = AsyncEventListener::new();

                listener.add_active_window_changed_handler(move |data| {
                    let mut output = output.clone();
                    Box::pin(async move {
                        if let Some(window) = data {
                            let _ = output.send(Message::ActiveWindowChanged(
                                Some(window.title),
                                Some(window.class),
                            )).await;
                        } else {
                            let _ = output.send(Message::ActiveWindowChanged(None, None)).await;
                        }
                    })
                });

                // Start the listener and keep it running
                if let Err(e) = listener.start_listener_async().await {
                    eprintln!("Hyprland window title listener error: {:?}", e);
                }

                // Keep the subscription alive indefinitely
                future::pending::<()>().await;
            }),
        )
    }
}
