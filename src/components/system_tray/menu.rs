//! Menu conversion and rendering utilities for the system tray.
//!
//! Converts SNI TrayMenu structures into a simplified format for Iced rendering.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Border, Color, Element, Length};
use system_tray::menu::{MenuItem as SniMenuItem, MenuType, ToggleState, TrayMenu};

/// Simplified menu item for Iced rendering.
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Menu item ID (used for activation)
    pub id: i32,
    /// Display label
    pub label: String,
    /// Whether the item is enabled/clickable
    pub enabled: bool,
    /// Whether this is a separator line
    pub is_separator: bool,
    /// Whether this item can be checked
    pub is_checkable: bool,
    /// Whether this item is currently checked
    pub is_checked: bool,
    /// Nested submenu items
    pub submenu: Vec<MenuItem>,
}

/// Convert an SNI TrayMenu to a list of simplified menu items.
pub fn convert_menu(menu: &TrayMenu) -> Vec<MenuItem> {
    menu.submenus.iter().map(convert_menu_item).collect()
}

/// Convert a single SNI menu item to our simplified format.
fn convert_menu_item(item: &SniMenuItem) -> MenuItem {
    let is_separator = matches!(item.menu_type, MenuType::Separator);
    let is_checked = matches!(item.toggle_state, ToggleState::On);
    let is_checkable = !matches!(
        item.toggle_type,
        system_tray::menu::ToggleType::CannotBeToggled
    );

    // Clean label: remove underscore access key markers (like _File -> File)
    let label = item
        .label
        .clone()
        .unwrap_or_default()
        .replace('_', "");

    MenuItem {
        id: item.id,
        label,
        enabled: item.enabled,
        is_separator,
        is_checkable,
        is_checked,
        submenu: item.submenu.iter().map(convert_menu_item).collect(),
    }
}

/// Message type for menu interactions.
#[derive(Debug, Clone)]
pub enum MenuMessage {
    /// A menu item was clicked
    ItemClicked(i32),
    /// Close the menu
    Close,
}

/// Render a menu as an Iced element.
pub fn render_menu<'a, M>(
    items: &'a [MenuItem],
    address: &'a str,
    on_item_click: impl Fn(String, i32) -> M + 'a + Clone,
    _on_close: M,
) -> Element<'a, M>
where
    M: Clone + 'a,
{
    let menu_items: Vec<Element<'_, M>> = items
        .iter()
        .filter(|item| !item.label.is_empty() || item.is_separator)
        .map(|item| render_menu_item(item, address, on_item_click.clone()))
        .collect();

    if menu_items.is_empty() {
        return Space::new(0, 0).into();
    }

    let menu_content = column(menu_items).spacing(0).width(Length::Fixed(200.0));

    container(menu_content)
        .padding(4)
        .style(|_theme| container::Style {
            background: Some(Color::from_rgba(0.1, 0.1, 0.1, 0.95).into()),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.3, 1.0),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// Render a single menu item.
fn render_menu_item<'a, M>(
    item: &'a MenuItem,
    address: &'a str,
    on_click: impl Fn(String, i32) -> M + 'a + Clone,
) -> Element<'a, M>
where
    M: Clone + 'a,
{
    if item.is_separator {
        return container(Space::new(Length::Fill, 1))
            .style(|_theme| container::Style {
                background: Some(Color::from_rgba(0.3, 0.3, 0.3, 0.5).into()),
                ..Default::default()
            })
            .width(Length::Fill)
            .padding([4, 0])
            .into();
    }

    let check_mark: Element<'_, M> = if item.is_checkable {
        text(if item.is_checked { "" } else { "  " })
            .size(12)
            .into()
    } else {
        Space::new(0, 0).into()
    };

    let content = row![check_mark, text(&item.label).size(13),]
        .spacing(4)
        .align_y(iced::Alignment::Center);

    let address_owned = address.to_string();
    let item_id = item.id;
    let on_click_clone = on_click.clone();

    let mut btn = button(content)
        .width(Length::Fill)
        .padding([4, 8])
        .style(move |_theme, status| menu_item_style(status, item.enabled));

    if item.enabled {
        btn = btn.on_press(on_click_clone(address_owned, item_id));
    }

    btn.into()
}

/// Style function for menu items.
fn menu_item_style(status: button::Status, enabled: bool) -> button::Style {
    let (background, text_color) = if !enabled {
        (None, Color::from_rgba(0.5, 0.5, 0.5, 1.0))
    } else {
        match status {
            button::Status::Hovered | button::Status::Pressed => (
                Some(Color::from_rgba(0.3, 0.3, 0.3, 0.8).into()),
                Color::WHITE,
            ),
            _ => (None, Color::WHITE),
        }
    };

    button::Style {
        background,
        text_color,
        border: Border::default(),
        shadow: Default::default(),
    }
}
