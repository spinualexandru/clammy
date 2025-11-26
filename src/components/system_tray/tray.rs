//! Main system tray component.
//!
//! Implements the StatusNotifierItem (SNI) protocol host for displaying
//! application tray icons and handling their interactions.

use std::collections::HashMap;
use std::sync::Arc;

use iced::futures::SinkExt;
use iced::stream;
use iced::widget::{button, container, image, text, tooltip, Row};
use iced::{Border, Color, Element, Length, Subscription, Task};
use std::future;
use system_tray::client::ActivateRequest;
use tokio::sync::mpsc;

use super::icon::{self, ICON_SIZE};
use super::menu::{self, MenuItem};

// ============================================================================
// Types
// ============================================================================

/// Internal representation of a tray item's state.
#[derive(Debug, Clone)]
struct TrayItemState {
    /// Unique identifier (D-Bus address)
    address: String,
    /// Human-readable title
    title: Option<String>,
    /// Cached icon handle for rendering
    icon_handle: Option<image::Handle>,
    /// Associated menu items
    menu_items: Vec<MenuItem>,
    /// Whether item only supports menu (no primary action)
    item_is_menu: bool,
}

/// Custom status indicator (not from SNI).
#[derive(Debug, Clone)]
pub struct CustomIndicator {
    /// Unique identifier
    pub id: String,
    /// Icon to display
    pub icon: image::Handle,
    /// Tooltip text
    pub tooltip: String,
}

/// The main SystemTray component state.
pub struct SystemTray {
    /// All tray items keyed by D-Bus address
    items: HashMap<String, TrayItemState>,
    /// Custom status indicators
    custom_indicators: Vec<CustomIndicator>,
    /// Currently open menu address (if any)
    open_menu: Option<String>,
    /// Channel sender for activation requests
    activate_tx: Option<mpsc::Sender<ActivateRequest>>,
}

/// Messages that the SystemTray component can handle.
#[derive(Debug, Clone)]
pub enum Message {
    /// SNI item was added
    ItemAdded {
        address: String,
        title: Option<String>,
        icon_handle: Option<image::Handle>,
        item_is_menu: bool,
    },
    /// SNI item was updated
    ItemUpdated {
        address: String,
        title: Option<String>,
        icon_handle: Option<image::Handle>,
    },
    /// SNI item menu was updated
    MenuUpdated {
        address: String,
        menu_items: Vec<MenuItem>,
    },
    /// SNI item was removed
    ItemRemoved(String),
    /// User left-clicked on a tray icon
    ItemClicked(String),
    /// User right-clicked on a tray icon
    ItemRightClicked(String),
    /// User clicked a menu item
    MenuItemClicked { address: String, menu_id: i32 },
    /// Close the open menu
    CloseMenu,
    /// Activation request completed
    ActivationComplete,
    /// Channel for sending activation requests
    ActivateChannelReady(mpsc::Sender<ActivateRequest>),
}

// ============================================================================
// Implementation
// ============================================================================

impl Default for SystemTray {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            custom_indicators: Vec::new(),
            open_menu: None,
            activate_tx: None,
        }
    }
}

impl std::fmt::Debug for SystemTray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemTray")
            .field("items", &self.items.len())
            .field("custom_indicators", &self.custom_indicators.len())
            .field("open_menu", &self.open_menu)
            .finish()
    }
}

impl SystemTray {
    /// Add a custom status indicator to the tray.
    pub fn add_custom_indicator(&mut self, indicator: CustomIndicator) {
        self.custom_indicators.push(indicator);
    }

    /// Remove a custom status indicator by ID.
    pub fn remove_custom_indicator(&mut self, id: &str) {
        self.custom_indicators.retain(|i| i.id != id);
    }

    /// Get menu items for a tray item by address.
    pub fn get_menu_items(&self, address: &str) -> Option<Vec<MenuItem>> {
        self.items.get(address).map(|item| item.menu_items.clone())
    }

    /// Check if an item has menu items or is menu-only.
    pub fn has_menu(&self, address: &str) -> bool {
        self.items
            .get(address)
            .map(|item| !item.menu_items.is_empty() || item.item_is_menu)
            .unwrap_or(false)
    }

    /// Update the component state based on received messages.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActivateChannelReady(tx) => {
                self.activate_tx = Some(tx);
                Task::none()
            }

            Message::ItemAdded {
                address,
                title,
                icon_handle,
                item_is_menu,
            } => {
                self.items.insert(
                    address.clone(),
                    TrayItemState {
                        address,
                        title,
                        icon_handle,
                        menu_items: Vec::new(),
                        item_is_menu,
                    },
                );
                Task::none()
            }

            Message::ItemUpdated {
                address,
                title,
                icon_handle,
            } => {
                if let Some(item) = self.items.get_mut(&address) {
                    if title.is_some() {
                        item.title = title;
                    }
                    if icon_handle.is_some() {
                        item.icon_handle = icon_handle;
                    }
                }
                Task::none()
            }

            Message::MenuUpdated {
                address,
                menu_items,
            } => {
                if let Some(item) = self.items.get_mut(&address) {
                    item.menu_items = menu_items;
                }
                Task::none()
            }

            Message::ItemRemoved(address) => {
                self.items.remove(&address);
                if self.open_menu.as_ref() == Some(&address) {
                    self.open_menu = None;
                }
                Task::none()
            }

            Message::ItemClicked(address) => {
                // Send activation request (menu handling is done by main.rs)
                if let Some(tx) = &self.activate_tx {
                    let tx = tx.clone();
                    Task::perform(
                        async move {
                            let _ = tx
                                .send(ActivateRequest::Default {
                                    address,
                                    x: 0,
                                    y: 0,
                                })
                                .await;
                        },
                        |_| Message::ActivationComplete,
                    )
                } else {
                    Task::none()
                }
            }

            Message::ItemRightClicked(address) => {
                if self.open_menu.as_ref() == Some(&address) {
                    self.open_menu = None;
                } else {
                    self.open_menu = Some(address);
                }
                Task::none()
            }

            Message::MenuItemClicked { address, menu_id } => {
                self.open_menu = None;

                if let Some(tx) = &self.activate_tx {
                    let tx = tx.clone();
                    Task::perform(
                        async move {
                            let _ = tx
                                .send(ActivateRequest::MenuItem {
                                    address,
                                    menu_path: "/MenuBar".to_string(),
                                    submenu_id: menu_id,
                                })
                                .await;
                        },
                        |_| Message::ActivationComplete,
                    )
                } else {
                    Task::none()
                }
            }

            Message::CloseMenu => {
                self.open_menu = None;
                Task::none()
            }

            Message::ActivationComplete => Task::none(),
        }
    }

    /// Render the system tray component.
    pub fn view(&self) -> Element<'_, Message> {
        // Collect SNI icons
        let sni_icons: Vec<Element<'_, Message>> = self
            .items
            .values()
            .map(|item| self.render_tray_item(item))
            .collect();

        // Collect custom indicator icons
        let custom_icons: Vec<Element<'_, Message>> = self
            .custom_indicators
            .iter()
            .map(|ind| self.render_custom_indicator(ind))
            .collect();

        // Combine all icons
        let all_icons: Vec<Element<'_, Message>> =
            sni_icons.into_iter().chain(custom_icons).collect();

        let icons_row = Row::from_vec(all_icons)
            .spacing(4)
            .align_y(iced::Alignment::Center);

        container(icons_row)
            .width(Length::Shrink)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .padding([0, 8])
            .into()
    }

    /// Render a single tray item.
    fn render_tray_item<'a>(&'a self, item: &'a TrayItemState) -> Element<'a, Message> {
        let icon_size = Length::Fixed(ICON_SIZE as f32);
        let is_menu_open = self.open_menu.as_ref() == Some(&item.address);

        let icon_element: Element<'_, Message> = if let Some(handle) = &item.icon_handle {
            image(handle.clone())
                .width(icon_size)
                .height(icon_size)
                .into()
        } else {
            // Fallback placeholder
            container(text("?").size(14))
                .width(icon_size)
                .height(icon_size)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        };

        let address = item.address.clone();

        // Tokyo Night colors
        let hover_bg = Color::from_rgba8(0x41, 0x48, 0x68, 0.5);  // Hover background
        let active_bg = Color::from_rgba8(0x41, 0x48, 0x68, 0.75); // Active/pressed background
        let text_color = Color::from_rgb8(0xc0, 0xca, 0xf5);       // Foreground

        let btn = button(icon_element)
            .padding(4)
            .style(move |_theme, status| {
                let bg = if is_menu_open {
                    Some(active_bg.into())
                } else {
                    match status {
                        button::Status::Hovered => Some(hover_bg.into()),
                        _ => None,
                    }
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
            .on_press(Message::ItemClicked(address));

        // Wrap with tooltip showing title
        if let Some(title) = &item.title {
            tooltip(btn, title.as_str(), tooltip::Position::Bottom).into()
        } else {
            btn.into()
        }
    }

    /// Render a custom status indicator.
    fn render_custom_indicator<'a>(&'a self, indicator: &'a CustomIndicator) -> Element<'a, Message> {
        let icon_size = Length::Fixed(ICON_SIZE as f32);

        let icon_element: Element<'_, Message> = image(indicator.icon.clone())
            .width(icon_size)
            .height(icon_size)
            .into();

        let btn = button(icon_element)
            .padding(2)
            .style(|_theme, _status| button::Style {
                background: None,
                border: Border::default(),
                text_color: Color::WHITE,
                shadow: Default::default(),
            });

        tooltip(btn, indicator.tooltip.as_str(), tooltip::Position::Bottom).into()
    }

    /// Subscribe to system tray events.
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run_with_id("system-tray-events", stream::channel(100, run_tray_client))
    }
}

/// Run the system tray client and forward events to messages.
async fn run_tray_client(mut output: iced::futures::channel::mpsc::Sender<Message>) {
    use system_tray::client::{Client, Event, UpdateEvent};

    // Create the SNI client
    let client = match Client::new().await {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("Failed to create system-tray client: {:?}", e);
            future::pending::<()>().await;
            return;
        }
    };

    // Create channel for activation requests
    let (activate_tx, mut activate_rx) = mpsc::channel::<ActivateRequest>(32);

    // Send the activation channel to the component
    let _ = output
        .send(Message::ActivateChannelReady(activate_tx))
        .await;

    // Subscribe to events
    let mut rx = client.subscribe();

    // Get and send initial items
    // Clone the data before releasing the lock to avoid holding MutexGuard across await
    let initial_items_data: Vec<_> = {
        let items_guard = client.items();
        let guard = items_guard.lock().unwrap();
        guard
            .iter()
            .map(|(addr, (item, menu))| {
                (
                    addr.clone(),
                    item.title.clone(),
                    icon::resolve_icon(item),
                    item.item_is_menu,
                    menu.as_ref().map(|m| menu::convert_menu(m)),
                )
            })
            .collect()
    };

    for (address, title, icon_handle, item_is_menu, menu_items_opt) in initial_items_data {
        let _ = output
            .send(Message::ItemAdded {
                address: address.clone(),
                title,
                icon_handle,
                item_is_menu,
            })
            .await;

        // If there's an initial menu, send that too
        if let Some(menu_items) = menu_items_opt {
            let _ = output
                .send(Message::MenuUpdated {
                    address,
                    menu_items,
                })
                .await;
        }
    }

    // Spawn activation handler
    let client_for_activate = Arc::clone(&client);
    tokio::spawn(async move {
        while let Some(request) = activate_rx.recv().await {
            if let Err(e) = client_for_activate.activate(request).await {
                eprintln!("Activation error: {:?}", e);
            }
        }
    });

    // Process events
    loop {
        match rx.recv().await {
            Ok(event) => match event {
                Event::Add(address, item) => {
                    let icon_handle = icon::resolve_icon(&item);
                    let _ = output
                        .send(Message::ItemAdded {
                            address,
                            title: item.title.clone(),
                            icon_handle,
                            item_is_menu: item.item_is_menu,
                        })
                        .await;
                }
                Event::Update(address, update) => match update {
                    UpdateEvent::Menu(menu) => {
                        let menu_items = menu::convert_menu(&menu);
                        let _ = output
                            .send(Message::MenuUpdated {
                                address,
                                menu_items,
                            })
                            .await;
                    }
                    UpdateEvent::Title(title) => {
                        let _ = output
                            .send(Message::ItemUpdated {
                                address,
                                title,
                                icon_handle: None,
                            })
                            .await;
                    }
                    _ => {
                        // For icon updates, we'd need to re-fetch the full item
                        // For now, we'll skip these
                    }
                },
                Event::Remove(address) => {
                    let _ = output.send(Message::ItemRemoved(address)).await;
                }
            },
            Err(e) => {
                eprintln!("System tray event error: {:?}", e);
                break;
            }
        }
    }

    future::pending::<()>().await;
}
