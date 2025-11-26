//! System tray component for displaying StatusNotifierItem (SNI) icons.
//!
//! This component provides:
//! - SNI protocol support for app icons (Discord, Steam, etc.)
//! - Right-click context menus
//! - Custom status indicator API

mod icon;
pub mod menu;
mod tray;

pub use tray::{Message, SystemTray};
