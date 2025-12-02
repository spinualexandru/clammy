use iced::futures::{SinkExt, Stream};
use iced::stream;
use iced::Color;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    // Font (None = system monospace)
    #[serde(default)]
    pub font: Option<String>,
    // Font size in pixels (default: 14)
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    // Spacing between tray widgets in pixels (default: 8)
    #[serde(default = "default_tray_widget_spacing")]
    pub tray_widget_spacing: f32,
    // Horizontal padding inside each tray widget in pixels (default: 8)
    #[serde(default = "default_tray_widget_padding")]
    pub tray_widget_padding: f32,

    // Core palette (used by Iced theme)
    pub background: String,
    pub background_alpha: f32,
    pub text: String,
    pub success: String,
    pub danger: String,

    // Extended colors (used by AppTheme methods)
    pub accent: String,
    pub accent2: String,
    pub info: String,
    pub surface: String,
    pub surface_alpha: f32,
    pub border: String,
    pub muted: String,
    pub hover: String,
    pub hover_alpha: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
        }
    }
}

fn default_font_size() -> f32 {
    14.0
}

fn default_tray_widget_spacing() -> f32 {
    8.0
}

fn default_tray_widget_padding() -> f32 {
    8.0
}

impl Default for ThemeConfig {
    fn default() -> Self {
        // Tokyo Night color scheme
        Self {
            font: None, // Uses system monospace
            font_size: default_font_size(),
            tray_widget_spacing: default_tray_widget_spacing(),
            tray_widget_padding: default_tray_widget_padding(),
            background: "#1a1b26".to_string(),
            background_alpha: 0.85,
            text: "#c0caf5".to_string(),
            success: "#9ece6a".to_string(),
            danger: "#f7768e".to_string(),
            accent: "#7aa2f7".to_string(),
            accent2: "#bb9af7".to_string(),
            info: "#7dcfff".to_string(),
            surface: "#24283b".to_string(),
            surface_alpha: 0.94,
            border: "#414868".to_string(),
            muted: "#565f89".to_string(),
            hover: "#414868".to_string(),
            hover_alpha: 0.5,
        }
    }
}

/// Get the config file path: $XDG_CONFIG_HOME/clammy/config.toml
pub fn config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("clammy");
    config_dir.join("config.toml")
}

impl Config {
    /// Load config from file, creating default if it doesn't exist
    pub fn load() -> Result<Self, ConfigError> {
        let path = config_path();

        if !path.exists() {
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(ConfigError::Io)?;
            }

            // Create default config and save it
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        // Read and parse existing config
        let content = fs::read_to_string(&path).map_err(ConfigError::Io)?;
        let config: Config = toml::from_str(&content).map_err(ConfigError::Parse)?;
        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = config_path();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ConfigError::Io)?;
        }

        let content = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;
        fs::write(&path, content).map_err(ConfigError::Io)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parse(e) => write!(f, "Parse error: {}", e),
            ConfigError::Serialize(e) => write!(f, "Serialize error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Parse a hex color string (e.g., "#7aa2f7") to iced::Color
pub fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Color::WHITE; // Fallback
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

    Color::from_rgb8(r, g, b)
}

/// Parse a hex color string with alpha
pub fn parse_hex_color_with_alpha(hex: &str, alpha: f32) -> Color {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Color::WHITE; // Fallback
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

    Color::from_rgba8(r, g, b, alpha)
}

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    Reloaded(Config),
    Error(String),
}

/// Subscription that watches the config file for changes
pub fn config_subscription() -> iced::Subscription<ConfigMessage> {
    iced::Subscription::run(config_watcher)
}

fn config_watcher() -> impl Stream<Item = ConfigMessage> {
    stream::channel(100, |mut output| async move {
        let path = config_path();
        let watch_path = path.parent().map(|p| p.to_path_buf()).unwrap_or(path.clone());

        // Create a channel for notify events
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(10);

        // Create the watcher
        let mut watcher: RecommendedWatcher = match notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                let _ = output
                    .send(ConfigMessage::Error(format!("Failed to create watcher: {}", e)))
                    .await;
                // Keep the task alive but do nothing
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                }
            }
        };

        // Start watching the config directory
        if let Err(e) = watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
            let _ = output
                .send(ConfigMessage::Error(format!(
                    "Failed to watch config: {}",
                    e
                )))
                .await;
        }

        // Process file change events
        loop {
            if let Some(event) = rx.recv().await {
                // Only reload on modify events for the config file
                if matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                ) {
                    // Check if this event is for our config file
                    let is_config_file = event.paths.iter().any(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n == "config.toml")
                            .unwrap_or(false)
                    });

                    if is_config_file {
                        // Small delay to ensure file is fully written
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                        match Config::load() {
                            Ok(config) => {
                                let _ = output.send(ConfigMessage::Reloaded(config)).await;
                            }
                            Err(e) => {
                                let _ = output
                                    .send(ConfigMessage::Error(format!(
                                        "Failed to reload config: {}",
                                        e
                                    )))
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    })
}
