use chrono::Local;
use iced::{Element, Subscription, time};

use super::tray_widget::tray_text;

#[derive(Debug, Clone)]
pub struct Clock {
    current_time: chrono::DateTime<Local>,
    formatted_buffer: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(chrono::DateTime<Local>),
}

impl Default for Clock {
    fn default() -> Self {
        let now = Local::now();
        Self {
            current_time: now,
            formatted_buffer: now.format("%a %d %b %H:%M").to_string(),
        }
    }
}

impl Clock {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick(time) => {
                self.current_time = time;
                // Reuse buffer - clear() doesn't deallocate capacity
                self.formatted_buffer.clear();
                use std::fmt::Write;
                let _ = write!(&mut self.formatted_buffer, "{}", time.format("%a %d %b %H:%M"));
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        tray_text(&self.formatted_buffer)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(1000)).map(|_| Message::Tick(Local::now()))
    }
}
