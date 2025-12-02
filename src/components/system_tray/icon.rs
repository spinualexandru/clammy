//! Icon handling utilities for the system tray.
//!
//! Handles:
//! - ARGB32 to RGBA conversion for SNI pixmap data
//! - Freedesktop icon theme lookup with caching
//! - Custom icon theme path resolution

use iced::widget::image;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use system_tray::item::{IconPixmap, StatusNotifierItem};

/// Default icon size for the tray (in pixels).
pub const ICON_SIZE: u16 = 22;

/// Cache for icon path lookups to avoid repeated filesystem checks.
/// Key: (theme_path, icon_name), Value: resolved path or None
static ICON_CACHE: RwLock<Option<HashMap<(String, String), Option<PathBuf>>>> = RwLock::new(None);

/// Initialize the icon cache if not already initialized.
fn get_or_init_cache() -> &'static RwLock<Option<HashMap<(String, String), Option<PathBuf>>>> {
    // Initialize on first access
    if let Ok(guard) = ICON_CACHE.read() {
        if guard.is_none() {
            drop(guard);
            if let Ok(mut guard) = ICON_CACHE.write() {
                if guard.is_none() {
                    *guard = Some(HashMap::new());
                }
            }
        }
    }
    &ICON_CACHE
}

/// Resolve an icon from an SNI item to an Iced image handle.
///
/// Resolution priority:
/// 1. Icon pixmap (raw ARGB32 data from the app)
/// 2. Icon name with custom theme path (cached)
/// 3. Icon name via freedesktop lookup
pub fn resolve_icon(item: &StatusNotifierItem) -> Option<image::Handle> {
    // Priority 1: Try icon pixmap (raw ARGB32 data)
    if let Some(pixmaps) = &item.icon_pixmap {
        if let Some(handle) = pixmap_to_handle(pixmaps) {
            return Some(handle);
        }
    }

    // Priority 2 & 3: Try icon name
    if let Some(icon_name) = &item.icon_name {
        if !icon_name.is_empty() {
            // Check custom theme path first
            if let Some(theme_path) = &item.icon_theme_path {
                if !theme_path.is_empty() {
                    if let Some(path) = find_icon_in_path_cached(theme_path, icon_name) {
                        return Some(image::Handle::from_path(path));
                    }
                }
            }

            // Fall back to freedesktop icon lookup
            if let Some(path) = lookup_freedesktop_icon(icon_name) {
                return Some(image::Handle::from_path(path));
            }
        }
    }

    None
}

/// Convert SNI ARGB32 pixmap data to an Iced RGBA image handle.
fn pixmap_to_handle(pixmaps: &[IconPixmap]) -> Option<image::Handle> {
    // Find the best size (closest to ICON_SIZE)
    let pixmap = pixmaps
        .iter()
        .filter(|p| p.width > 0 && p.height > 0)
        .min_by_key(|p| (p.width - ICON_SIZE as i32).abs())?;

    if pixmap.pixels.is_empty() {
        return None;
    }

    // Convert ARGB32 (network byte order) to RGBA
    let rgba = argb32_to_rgba(&pixmap.pixels, pixmap.width as usize, pixmap.height as usize);

    Some(image::Handle::from_rgba(
        pixmap.width as u32,
        pixmap.height as u32,
        rgba,
    ))
}

/// Convert ARGB32 (big-endian/network byte order) to RGBA.
///
/// SNI icons use ARGB32 format in network byte order: [A, R, G, B]
/// Iced expects RGBA format: [R, G, B, A]
fn argb32_to_rgba(argb: &[u8], width: usize, height: usize) -> Vec<u8> {
    let expected_len = width * height * 4;
    if argb.len() < expected_len {
        // Return transparent pixels if data is invalid
        return vec![0; expected_len];
    }

    let mut rgba = Vec::with_capacity(expected_len);

    for chunk in argb.chunks_exact(4) {
        // ARGB32 in network byte order: [A, R, G, B]
        let a = chunk[0];
        let r = chunk[1];
        let g = chunk[2];
        let b = chunk[3];

        // Output RGBA: [R, G, B, A]
        rgba.push(r);
        rgba.push(g);
        rgba.push(b);
        rgba.push(a);
    }

    rgba
}

/// Look up an icon using the freedesktop icon theme specification.
///
/// Note: Freedesktop icon lookup has been disabled to reduce memory usage.
/// Most apps provide icon pixmaps or custom theme paths, so this fallback
/// is rarely needed. If an icon doesn't appear, the app should provide pixmap data.
fn lookup_freedesktop_icon(_name: &str) -> Option<PathBuf> {
    None // Disabled for memory optimization
}

/// Find an icon in a custom theme path with caching.
fn find_icon_in_path_cached(theme_path: &str, icon_name: &str) -> Option<PathBuf> {
    let cache = get_or_init_cache();
    let key = (theme_path.to_string(), icon_name.to_string());

    // Check cache first
    if let Ok(guard) = cache.read() {
        if let Some(cache_map) = guard.as_ref() {
            if let Some(cached) = cache_map.get(&key) {
                return cached.clone();
            }
        }
    }

    // Not in cache, perform lookup
    let result = find_icon_in_path(theme_path, icon_name);

    // Store in cache
    if let Ok(mut guard) = cache.write() {
        if let Some(cache_map) = guard.as_mut() {
            cache_map.insert(key, result.clone());
        }
    }

    result
}

/// Find an icon in a custom theme path provided by the SNI item.
fn find_icon_in_path(theme_path: &str, icon_name: &str) -> Option<PathBuf> {
    let extensions = ["png", "svg", "xpm"];
    let sizes: [u16; 6] = [ICON_SIZE, 24, 32, 48, 22, 16];

    // Try size-specific directories
    for size in sizes {
        for ext in &extensions {
            let path = PathBuf::from(theme_path)
                .join(format!("{size}x{size}"))
                .join(format!("{icon_name}.{ext}"));
            if path.exists() {
                return Some(path);
            }
        }
    }

    // Try direct path without size directory
    for ext in &extensions {
        let path = PathBuf::from(theme_path).join(format!("{icon_name}.{ext}"));
        if path.exists() {
            return Some(path);
        }
    }

    // Try hicolor theme structure
    for size in sizes {
        for ext in &extensions {
            let path = PathBuf::from(theme_path)
                .join("hicolor")
                .join(format!("{size}x{size}"))
                .join("apps")
                .join(format!("{icon_name}.{ext}"));
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}
