mod components;
mod config;
mod hyprland_events;
mod styles;
mod theme;

use std::collections::HashMap;

use iced::event::{self, Event};
use iced::keyboard::{self, key::Named};
use iced::border::Radius;
use iced::widget::container::Style;
use iced::widget::{button, column, container, row, text};
use iced::window::Id;
use iced::{Border, Element, Font, Length, Subscription, Task};
use iced_layershell::actions::{IcedNewMenuSettings, MenuDirection};
use iced_layershell::build_pattern::{MainSettings, daemon};
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::LayerShellSettings;
use iced_layershell::to_layer_message;

use crate::config::{Config, ConfigMessage, config_subscription};
use crate::theme::{AppTheme, set_global_theme};
use components::battery;
use components::clock;
use components::notification_toggle;
use components::system_tray;
use components::window_title;
use components::workspaces;

pub fn main() -> Result<(), iced_layershell::Error> {
    // Load config early to get font setting
    let config = Config::load().unwrap_or_default();
    let default_font = match &config.theme.font {
        Some(name) => Font::with_name(Box::leak(name.clone().into_boxed_str())),
        None => Font::MONOSPACE,
    };

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
        default_font,
        antialiasing: true,
        ..MainSettings::default()
    })
    .run_with(StatusBar::new)
}

/// Window type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowType {
    Main,
    TrayMenu,
}

/// Animation state for dropdown menus
#[derive(Debug, Clone)]
struct PopupAnimationState {
    /// Progress from 0.0 (closed) to 1.0 (fully open)
    progress: f32,
    /// Total height of menu content
    content_height: f32,
}

struct StatusBar {
    config: Config,
    app_theme: AppTheme,
    battery: battery::Battery,
    clock: clock::Clock,
    notification_toggle: notification_toggle::NotificationToggle,
    workspaces: workspaces::Workspaces,
    window_title: window_title::WindowTitle,
    system_tray: system_tray::SystemTray,
    /// Track window IDs and their types
    windows: HashMap<Id, WindowType>,
    /// Store menu data for popup windows (keyed by popup ID)
    menu_data: HashMap<Id, (String, Vec<system_tray::menu::MenuItem>)>,
    /// Animation state for popup windows
    popup_animations: HashMap<Id, PopupAnimationState>,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Message {
    Battery(battery::Message),
    Clock(clock::Message),
    NotificationToggle(notification_toggle::Message),
    Workspaces(workspaces::Message),
    WindowTitle(window_title::Message),
    SystemTray(system_tray::Message),
    /// Config file changed - hot reload
    ConfigChanged(ConfigMessage),
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
    /// Animation tick for popup slide-down
    PopupAnimationTick,
}

impl StatusBar {
    fn new() -> (Self, Task<Message>) {
        // Load config (creates default if missing)
        let config = Config::load().unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}, using defaults", e);
            Config::default()
        });
        let app_theme = AppTheme::from_config(&config);

        // Set global theme for component access
        set_global_theme(&app_theme);

        (
            Self {
                config,
                app_theme,
                battery: battery::Battery::default(),
                clock: clock::Clock::default(),
                notification_toggle: notification_toggle::NotificationToggle::default(),
                workspaces: workspaces::Workspaces::default(),
                window_title: window_title::WindowTitle::default(),
                system_tray: system_tray::SystemTray::default(),
                windows: HashMap::new(),
                menu_data: HashMap::new(),
                popup_animations: HashMap::new(),
            },
            Task::done(workspaces::Message::Refresh).map(Message::Workspaces),
        )
    }

    fn namespace(&self) -> String {
        String::from("clammy")
    }

    fn theme(&self) -> iced::Theme {
        (&self.app_theme).into()
    }

    fn remove_id(&mut self, id: Id) {
        if let Some(window_type) = self.windows.remove(&id) {
            if matches!(window_type, WindowType::TrayMenu) {
                self.menu_data.remove(&id);
                self.popup_animations.remove(&id);
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Battery(msg) => self.battery.update(msg).map(Message::Battery),
            Message::Clock(msg) => {
                self.clock.update(msg);
                Task::none()
            }
            Message::NotificationToggle(msg) => {
                self.notification_toggle.update(msg).map(Message::NotificationToggle)
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
            Message::ConfigChanged(config_msg) => {
                match config_msg {
                    ConfigMessage::Reloaded(new_config) => {
                        self.config = new_config;
                        self.app_theme.update(&self.config);
                        set_global_theme(&self.app_theme);
                    }
                    ConfigMessage::Error(e) => {
                        eprintln!("Config error: {}", e);
                    }
                }
                Task::none()
            }
            Message::OpenTrayMenu { address, items } => {
                // Create popup window
                let id = Id::unique();

                // Calculate menu height (roughly 28px per item + padding)
                let item_count = items.len();
                let menu_height = (item_count as u32 * 28) + 16;
                // Add 18px top offset + 4px connector height
                let height = menu_height + 22;
                let content_height = menu_height as f32;

                // Store menu data keyed by popup ID
                self.menu_data.insert(id, (address, items));
                self.windows.insert(id, WindowType::TrayMenu);

                // Initialize animation state - starts at 0.0
                self.popup_animations.insert(
                    id,
                    PopupAnimationState {
                        progress: 0.0,
                        content_height,
                    },
                );

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
                let tray_msg = system_tray::Message::MenuItemClicked { address, menu_id };
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
                        .find(|(_, wt)| matches!(wt, WindowType::TrayMenu))
                    {
                        return Task::done(Message::ClosePopup(id));
                    }
                }
                Task::none()
            }
            Message::PopupAnimationTick => {
                // Find the first animating popup and advance it
                if let Some((_, anim)) = self
                    .popup_animations
                    .iter_mut()
                    .find(|(_, a)| a.progress < 1.0)
                {
                    // Ease-out quadratic for smoother animation
                    anim.progress = (anim.progress + 0.15).min(1.0);
                }
                Task::none()
            }
            _ => Task::none(), // Handle layer shell messages
        }
    }

    fn view(&self, id: Id) -> Element<'_, Message> {
        match self.windows.get(&id) {
            Some(WindowType::TrayMenu) => self.view_tray_menu(id),
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
        let battery = self.battery.view().map(Message::Battery);
        let clock = self.clock.view().map(Message::Clock);
        let notification_toggle = self.notification_toggle.view().map(Message::NotificationToggle);
        let right = row![system_tray, battery, clock, notification_toggle]
            .spacing(self.app_theme.tray_widget_spacing())
            .align_y(iced::Alignment::Center);

        let content = row![left, middle, right,]
            .padding(5)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill);

        let accent = self.app_theme.accent();

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

    fn view_tray_menu(&self, popup_id: Id) -> Element<'_, Message> {
        let (address, items) = match self.menu_data.get(&popup_id) {
            Some(data) => data,
            None => {
                return container(text("Menu not found"))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
            }
        };

        // Get animation progress (default to 1.0 = fully visible)
        let (progress, content_height) = self
            .popup_animations
            .get(&popup_id)
            .map(|anim| {
                // Ease-out quadratic for smoother feel
                let eased = 1.0 - (1.0 - anim.progress).powi(2);
                (eased, anim.content_height)
            })
            .unwrap_or((1.0, 100.0));

        let border_color = self.app_theme.border();
        let hover_color = self.app_theme.hover();
        let text_color = self.app_theme.text();
        let muted_color = self.app_theme.muted();
        let surface_color = self.app_theme.surface();
        let accent_color = self.app_theme.accent();
        let font_size = self.app_theme.font_size();

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

                    let label_widget = if item.is_checkable && item.is_checked {
                        text(format!(" {}", item.label)).size(font_size)
                    } else {
                        text(&item.label).size(font_size)
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
                                text_color: if enabled { text_color } else { muted_color },
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

        let menu_column = column(menu_items).spacing(0).width(Length::Fill);

        // Animated height - clip content by showing only a portion
        let visible_height = (content_height * progress).max(1.0);

        // Small connector tab at top to bridge gap with status bar
        let connector = container(iced::widget::Space::new(Length::Fill, 0))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(4.0))
            .style(move |_theme| container::Style {
                background: Some(accent_color.into()),
                border: Border {
                    radius: Radius {
                        top_left: 2.0,
                        top_right: 2.0,
                        bottom_left: 0.0,
                        bottom_right: 0.0,
                    },
                    ..Border::default()
                },
                ..Default::default()
            });

        // Menu content container with clipped height for animation
        let menu_container = container(menu_column)
            .width(Length::Fill)
            .height(Length::Fixed(visible_height))
            .clip(true)
            .padding(4)
            .style(move |_theme| container::Style {
                background: Some(surface_color.into()),
                border: Border {
                    color: accent_color,
                    width: 1.0,
                    radius: Radius {
                        top_left: 6.0,
                        top_right: 6.0,
                        bottom_left: 6.0,
                        bottom_right: 6.0,
                    },
                },
                ..Default::default()
            });

        // Add top spacing to offset from bar center to bar bottom
        // Bar is 36px, menu appears at center (18px), so add ~18px offset
        let top_spacer = iced::widget::Space::new(Length::Fill, Length::Fixed(18.0));

        // Stack: spacer, connector, menu
        let content = column![
            top_spacer,
            container(connector).width(Length::Fill).center_x(Length::Fill),
            menu_container,
        ]
        .spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Animation subscription only active when a popup is animating
        let has_animating = self
            .popup_animations
            .values()
            .any(|anim| anim.progress < 1.0);

        let animation_subscription = if has_animating {
            iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| Message::PopupAnimationTick)
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![
            self.battery.subscription().map(Message::Battery),
            self.clock.subscription().map(Message::Clock),
            self.notification_toggle.subscription().map(Message::NotificationToggle),
            self.workspaces.subscription().map(Message::Workspaces),
            self.window_title.subscription().map(Message::WindowTitle),
            self.system_tray.subscription().map(Message::SystemTray),
            config_subscription().map(Message::ConfigChanged),
            event::listen().map(Message::IcedEvent),
            animation_subscription,
        ])
    }
}
