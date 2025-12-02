use iced::widget::{container, text};
use iced::{Element, Subscription, Task, time};
use std::fs;
use std::path::PathBuf;

use super::tray_widget::tray_text;

const BATTERY_PATH: &str = "/sys/class/power_supply/BAT0";

#[derive(Debug, Clone)]
pub struct Battery {
    percentage: Option<u8>,
    charging: bool,
    display_text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

impl Default for Battery {
    fn default() -> Self {
        let (percentage, charging) = read_battery_info();
        let mut battery = Self {
            percentage,
            charging,
            display_text: String::new(),
        };
        battery.update_display();
        battery
    }
}

impl Battery {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let (percentage, charging) = read_battery_info();
                self.percentage = percentage;
                self.charging = charging;
                self.update_display();
                Task::none()
            }
        }
    }

    fn update_display(&mut self) {
        self.display_text.clear();
        if let Some(pct) = self.percentage {
            let icon = self.get_icon(pct);
            use std::fmt::Write;
            let _ = write!(&mut self.display_text, "{} {}%", icon, pct);
        }
    }

    fn get_icon(&self, percentage: u8) -> &'static str {
        if self.charging {
            return "󰂄"; // nf-md-battery_charging
        }
        match percentage {
            90..=100 => "󰁹", // nf-md-battery
            80..=89 => "󰂂",  // nf-md-battery_80
            70..=79 => "󰂁",  // nf-md-battery_70
            60..=69 => "󰂀",  // nf-md-battery_60
            50..=59 => "󰁿",  // nf-md-battery_50
            40..=49 => "󰁾",  // nf-md-battery_40
            30..=39 => "󰁽",  // nf-md-battery_30
            20..=29 => "󰁼",  // nf-md-battery_20
            10..=19 => "󰁻",  // nf-md-battery_10
            _ => "󰂃",        // nf-md-battery_alert (0-9%)
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Hide if no battery present
        if self.percentage.is_none() {
            return container(text("")).into();
        }

        tray_text(&self.display_text)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Update every 30 seconds (battery changes slowly)
        time::every(std::time::Duration::from_secs(30)).map(|_| Message::Tick)
    }
}

/// Read battery info from sysfs, reusing PathBuf to minimize allocations
fn read_battery_info() -> (Option<u8>, bool) {
    let mut path = PathBuf::from(BATTERY_PATH);

    if !path.exists() {
        return (None, false);
    }

    // Read capacity
    path.push("capacity");
    let capacity = fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok());

    // Read status (reuse path)
    path.pop();
    path.push("status");
    let charging = fs::read_to_string(&path)
        .map(|s| s.trim() == "Charging")
        .unwrap_or(false);

    (capacity, charging)
}
