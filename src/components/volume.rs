use iced::{Element, Subscription, Task, time};
use std::process::Command;

use super::tray_widget::tray_text;

#[derive(Debug, Clone)]
pub struct Volume {
    percentage: u8,
    muted: bool,
    display_text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

impl Default for Volume {
    fn default() -> Self {
        let (percentage, muted) = read_volume_info();
        let mut volume = Self {
            percentage,
            muted,
            display_text: String::new(),
        };
        volume.update_display();
        volume
    }
}

impl Volume {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                let (percentage, muted) = read_volume_info();
                self.percentage = percentage;
                self.muted = muted;
                self.update_display();
                Task::none()
            }
        }
    }

    fn update_display(&mut self) {
        self.display_text.clear();
        let icon = self.get_icon();
        use std::fmt::Write;
        let _ = write!(&mut self.display_text, "{} {}%", icon, self.percentage);
    }

    fn get_icon(&self) -> &'static str {
        if self.muted {
            return "󰝟"; // nf-md-volume_off
        }
        match self.percentage {
            66..=100 => "󰕾", // nf-md-volume_high
            33..=65 => "󰖀",  // nf-md-volume_medium
            _ => "󰕿",        // nf-md-volume_low
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        tray_text(&self.display_text)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Update every 2 seconds
        time::every(std::time::Duration::from_secs(2)).map(|_| Message::Tick)
    }
}

fn read_volume_info() -> (u8, bool) {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Expected format: "Volume: 0.45" or "Volume: 0.45 [MUTED]"
            
            let muted = stdout.contains("[MUTED]");
            
            // Extract the float value
            if let Some(vol_str) = stdout.split_whitespace().nth(1) {
                if let Ok(vol_float) = vol_str.parse::<f32>() {
                     return ((vol_float * 100.0) as u8, muted);
                }
            }
            (0, false)
        }
        Err(_) => (0, false), // Fail gracefully
    }
}
