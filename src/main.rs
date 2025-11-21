mod components;
mod theme;

use iced::widget::container::Style;
use iced::widget::{container, row};
use iced::{Element, Font, Length, Subscription, Task, Theme};
// keep alias for any other uses
use iced_layershell::actions::LayershellCustomActions;
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::Application;

use crate::theme::AppTheme;
use components::clock;
use components::window_title;
use components::workspaces;

pub fn main() -> Result<(), iced_layershell::Error> {
    let settings = Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            layer: Layer::Top,
            exclusive_zone: 36,
            size: Some((0, 36)),
            margin: (0, 0, 15, 0),
            ..LayerShellSettings::default()
        },
        default_font: Font::with_name("IBM Plex Mono"),
        antialiasing: true,
        ..Settings::default()
    };
    StatusBar::run(settings)
}

struct StatusBar {
    theme: AppTheme,
    clock: clock::Clock,
    workspaces: workspaces::Workspaces,
    window_title: window_title::WindowTitle,

}

#[derive(Debug, Clone)]
enum Message {
    Clock(clock::Message),
    Workspaces(workspaces::Message),
    WindowTitle(window_title::Message),
}

impl TryFrom<Message> for LayershellCustomActions {
    type Error = Message;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        Err(message)
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            clock: clock::Clock::default(),
            workspaces: workspaces::Workspaces::default(),
            window_title: window_title::WindowTitle::default(),
        }
    }
}

impl Application for StatusBar {
    type Executor = iced::executor::Default;
    type Message = Message;
    // Use the iced built-in Theme for compatibility with iced_layershell
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::done(workspaces::Message::Refresh).map(Message::Workspaces),
        )
    }

    fn theme(&self) -> Self::Theme {
        // convert our AppTheme into an iced::Theme
        self.theme.into()
    }

    fn namespace(&self) -> String {
        String::from("clammy")
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let left = self.workspaces.view().map(Message::Workspaces);

        let middle = container(self.window_title.view().map(Message::WindowTitle))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .style(|_theme| Style::default());

        let right = self.clock.view().map(Message::Clock);

        let content = row![
            left,
            middle,
            right,
        ]
            .padding(5)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| Style::default())
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            self.clock.subscription().map(Message::Clock),
            self.workspaces.subscription().map(Message::Workspaces),
            self.window_title.subscription().map(Message::WindowTitle),
        ])
    }
}
