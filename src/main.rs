mod components;
mod theme;

use std::collections::HashMap;

use iced::event::{self, Event};
use iced::keyboard::{self, key::Named};
use iced::widget::container::Style;
use iced::widget::{button, column, container, row, text};
use iced::window::Id;
use iced::{Border, Element, Font, Length, Subscription, Task};
use iced_layershell::actions::{IcedNewMenuSettings, MenuDirection};
use iced_layershell::build_pattern::{daemon, MainSettings};
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::LayerShellSettings;
use iced_layershell::to_layer_message;

use crate::theme::AppTheme;
use components::clock;
use components::system_tray;
use components::window_title;
use components::workspaces;

pub fn main() -> Result<(), iced_layershell::Error> {
    daemon(
        StatusBar::namespace,
        StatusBar::update,
        StatusBar::view,
        StatusBar::remove_id,
    )
    .subscription(StatusBar::subscription)
    .theme(StatusBar::theme)
    .settings(MainSettings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            layer: Layer::Top,
            exclusive_zone: 36,
            size: Some((0, 36)),
            margin: (4, 4, 15, 4),
            ..LayerShellSettings::default()
        },
        default_font: Font::with_name("IBM Plex Mono"),
        antialiasing: true,
        ..MainSettings::default()
    })
    .run_with(StatusBar::new)
}

/// Window type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowType {
    Main,
    TrayMenu(usize), // Index into menu_data
}

struct StatusBar {
    app_theme: AppTheme,
    clock: clock::Clock,
    workspaces: workspaces::Workspaces,
    window_title: window_title::WindowTitle,
    system_tray: system_tray::SystemTray,
    /// Track window IDs and their types
    windows: HashMap<Id, WindowType>,
    /// Store menu data for popup windows
    menu_data: Vec<(String, Vec<system_tray::menu::MenuItem>)>, // (address, items)
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Message {
    Clock(clock::Message),
    Workspaces(workspaces::Message),
    WindowTitle(window_title::Message),
    SystemTray(system_tray::Message),
    /// Open a tray menu popup
    OpenTrayMenu {
        address: String,
        items: Vec<system_tray::menu::MenuItem>,
    },
    /// Close a popup window
    ClosePopup(Id),
    /// Menu item was clicked in popup
    PopupMenuItemClicked {
        popup_id: Id,
        address: String,
        menu_id: i32,
    },
    /// Global event for keyboard/mouse handling
    IcedEvent(Event),
}

impl StatusBar {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                app_theme: AppTheme::default(),
                clock: clock::Clock::default(),
                workspaces: workspaces::Workspaces::default(),
                window_title: window_title::WindowTitle::default(),
                system_tray: system_tray::SystemTray::default(),
                windows: HashMap::new(),
                menu_data: Vec::new(),
            },
            Task::done(workspaces::Message::Refresh).map(Message::Workspaces),
        )
    }

    fn namespace(&self) -> String {
        String::from("clammy")
    }

    fn theme(&self) -> iced::Theme {
        self.app_theme.into()
    }

    fn remove_id(&mut self, id: Id) {
        if let Some(WindowType::TrayMenu(_idx)) = self.windows.remove(&id) {
            // Clean up menu data if this was the last reference
            // For simplicity, we'll just leave it (could optimize later)
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Clock(msg) => {
                self.clock.update(msg);
                Task::none()
            }
            Message::Workspaces(msg) => self.workspaces.update(msg).map(Message::Workspaces),
            Message::WindowTitle(msg) => {
                self.window_title.update(msg);
                Task::none()
            }
            Message::SystemTray(msg) => {
                // Check if this is a menu open request
                if let system_tray::Message::ItemClicked(ref address) = msg {
                    if let Some(items) = self.system_tray.get_menu_items(address) {
                        if !items.is_empty() {
                            return Task::done(Message::OpenTrayMenu {
                                address: address.clone(),
                                items,
                            });
                        }
                    }
                }
                self.system_tray.update(msg).map(Message::SystemTray)
            }
            Message::OpenTrayMenu { address, items } => {
                // Store menu data
                let menu_idx = self.menu_data.len();
                self.menu_data.push((address, items));

                // Create popup window
                let id = Id::unique();
                self.windows.insert(id, WindowType::TrayMenu(menu_idx));

                // Calculate menu height (roughly 24px per item + padding)
                let item_count = self.menu_data[menu_idx].1.len();
                let height = (item_count as u32 * 28) + 16;

                Task::done(Message::NewMenu {
                    settings: IcedNewMenuSettings {
                        size: (200, height.min(400)),
                        direction: MenuDirection::Down,
                    },
                    id,
                })
            }
            Message::ClosePopup(id) => {
                self.remove_id(id);
                Task::done(Message::RemoveWindow(id))
            }
            Message::PopupMenuItemClicked {
                popup_id,
                address,
                menu_id,
            } => {
                // Forward to system tray and close popup
                let tray_msg = system_tray::Message::MenuItemClicked {
                    address,
                    menu_id,
                };
                let close_task = Task::done(Message::ClosePopup(popup_id));
                let tray_task = self.system_tray.update(tray_msg).map(Message::SystemTray);
                Task::batch([close_task, tray_task])
            }
            Message::IcedEvent(event) => {
                // Handle ESC key to close any open popup
                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Escape),
                    ..
                }) = event
                {
                    // Find and close any TrayMenu windows
                    if let Some((&id, _)) = self
                        .windows
                        .iter()
                        .find(|(_, wt)| matches!(wt, WindowType::TrayMenu(_)))
                    {
                        return Task::done(Message::ClosePopup(id));
                    }
                }
                Task::none()
            }
            _ => Task::none(), // Handle layer shell messages
        }
    }

    fn view(&self, id: Id) -> Element<'_, Message> {
        match self.windows.get(&id) {
            Some(WindowType::TrayMenu(menu_idx)) => self.view_tray_menu(id, *menu_idx),
            _ => self.view_main(),
        }
    }

    fn view_main(&self) -> Element<'_, Message> {
        let left = self.workspaces.view().map(Message::Workspaces);

        let middle = container(self.window_title.view().map(Message::WindowTitle))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .style(|_theme| Style::default());

        let system_tray = self.system_tray.view().map(Message::SystemTray);
        let clock = self.clock.view().map(Message::Clock);
        let right = row![system_tray, clock]
            .spacing(8)
            .align_y(iced::Alignment::Center);

        let content = row![left, middle, right,]
            .padding(5)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill);

        let theme = self.app_theme;
        let accent = theme.accent();

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |theme: &iced::Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(palette.primary.into()),
                    border: Border {
                        radius: 15.0.into(),
                        width: 1.0.into(),
                        color: accent,
                        ..Border::default()
                    },
                    ..container::Style::default()
                }
            })
            .into()
    }

    fn view_tray_menu(&self, popup_id: Id, menu_idx: usize) -> Element<'_, Message> {
        let (address, items) = match self.menu_data.get(menu_idx) {
            Some(data) => data,
            None => {
                return container(text("Menu not found"))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
            }
        };

        let theme = self.app_theme;
        let border_color = theme.border();
        let hover_color = theme.hover();
        let text_color = theme.text();
        let muted_color = theme.muted();
        let surface_color = theme.surface();

        let menu_items: Vec<Element<'_, Message>> = items
            .iter()
            .filter(|item| !item.label.is_empty() || item.is_separator)
            .map(|item| {
                if item.is_separator {
                    container(iced::widget::Space::new(Length::Fill, 1))
                        .style(move |_theme| container::Style {
                            background: Some(border_color.into()),
                            ..Default::default()
                        })
                        .width(Length::Fill)
                        .padding([4, 0])
                        .into()
                } else {
                    let addr = address.clone();
                    let item_id = item.id;
                    let enabled = item.enabled;

                    // Use the label directly to avoid lifetime issues
                    let label_widget = if item.is_checkable && item.is_checked {
                        text(format!(" {}", item.label)).size(13)
                    } else {
                        text(&item.label).size(13)
                    };

                    let mut btn = button(label_widget)
                        .width(Length::Fill)
                        .padding([6, 12])
                        .style(move |_theme, status| {
                            let bg = if !enabled {
                                None
                            } else {
                                match status {
                                    button::Status::Hovered | button::Status::Pressed => {
                                        Some(hover_color.into())
                                    }
                                    _ => None,
                                }
                            };
                            button::Style {
                                background: bg,
                                text_color: if enabled {
                                    text_color
                                } else {
                                    muted_color
                                },
                                border: Border::default(),
                                shadow: Default::default(),
                            }
                        });

                    if enabled {
                        btn = btn.on_press(Message::PopupMenuItemClicked {
                            popup_id,
                            address: addr,
                            menu_id: item_id,
                        });
                    }

                    btn.into()
                }
            })
            .collect();

        container(column(menu_items).spacing(0).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(4)
            .style(move |_theme| container::Style {
                background: Some(surface_color.into()),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            self.clock.subscription().map(Message::Clock),
            self.workspaces.subscription().map(Message::Workspaces),
            self.window_title.subscription().map(Message::WindowTitle),
            self.system_tray.subscription().map(Message::SystemTray),
            event::listen().map(Message::IcedEvent),
        ])
    }
}
