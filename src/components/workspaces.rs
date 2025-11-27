//! Workspaces component for displaying and managing Hyprland workspaces.
//!
//! This component provides:
//! - Real-time workspace list display
//! - Active workspace highlighting
//! - Click-to-switch functionality
//! - Automatic updates via Hyprland event subscription

use hyprland::data::{Workspace, Workspaces as HyprWorkspaces};
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::event_listener::AsyncEventListener;
use hyprland::shared::{HyprData, HyprDataActive, WorkspaceId};
use iced::futures::SinkExt;
use iced::stream;
use iced::widget::{Row, button, container, text};
use iced::{Border, Color, Element, Length, Subscription, Task};
use std::future;

// ============================================================================
// Types
// ============================================================================

/// The main Workspaces component state.
#[derive(Debug, Clone)]
pub struct Workspaces {
    /// List of all available workspaces
    workspaces: Vec<WorkspaceInfo>,
    /// ID of the currently active workspace
    active_workspace_id: Option<WorkspaceId>,
}

/// Simplified workspace information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceInfo {
    pub(crate) id: WorkspaceId,
    pub(crate) name: String,
    pub(crate) monitor: String,
    pub(crate) windows: u16,
    id_string: String,  // Cached for rendering
}

/// Messages that the Workspaces component can handle.
#[derive(Debug, Clone)]
pub enum Message {
    /// Trigger a refresh of workspace data from Hyprland
    Refresh,
    /// Update internal state with new workspace data
    #[doc(hidden)]
    WorkspacesUpdated {
        workspaces: Vec<WorkspaceInfo>,
        active_id: Option<WorkspaceId>,
    },
    /// User clicked on a workspace to switch to it
    WorkspaceClicked(WorkspaceId),
    /// Workspace switch operation completed
    #[doc(hidden)]
    WorkspaceSwitched,
}

// ============================================================================
// Implementation
// ============================================================================

impl Default for Workspaces {
    fn default() -> Self {
        Self {
            workspaces: Vec::new(),
            active_workspace_id: None,
        }
    }
}

impl Workspaces {
    /// Update the component state based on received messages.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                // Fetch workspace data asynchronously
                Task::perform(Self::fetch_workspace_data(), |result| {
                    Message::WorkspacesUpdated {
                        workspaces: result.0,
                        active_id: result.1,
                    }
                })
            }

            Message::WorkspacesUpdated {
                workspaces,
                active_id,
            } => {
                self.workspaces = workspaces;
                self.active_workspace_id = active_id;
                Task::none()
            }

            Message::WorkspaceClicked(workspace_id) => {
                // Switch to the clicked workspace
                Task::perform(Self::switch_workspace(workspace_id), |_| {
                    Message::WorkspaceSwitched
                })
            }

            Message::WorkspaceSwitched => {
                // Refresh workspace list after switching
                Task::done(Message::Refresh)
            }
        }
    }

    /// Render the workspaces component.
    pub fn view(&self) -> Element<'_, Message> {
        let workspace_buttons = self.create_workspace_buttons();

        let content = workspace_buttons
            .spacing(8)
            .padding([0, 8])
            .align_y(iced::Alignment::Center);

        container(content)
            .width(Length::Shrink)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    /// Subscribe to Hyprland workspace events.
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run_with_id(
            "hyprland-workspace-events",
            stream::channel(100, |mut output| async move {
                Self::run_event_listener(&mut output).await;
                future::pending::<()>().await;
            }),
        )
    }

    // ------------------------------------------------------------------------
    // Private helper methods
    // ------------------------------------------------------------------------

    /// Fetch workspace data from Hyprland.
    async fn fetch_workspace_data() -> (Vec<WorkspaceInfo>, Option<WorkspaceId>) {
        let workspaces = match HyprWorkspaces::get() {
            Ok(ws) => {
                let mut info: Vec<WorkspaceInfo> = ws
                    .into_iter()
                    .map(|w| WorkspaceInfo {
                        id: w.id,
                        id_string: w.id.to_string(),  // Cache once
                        name: w.name,
                        monitor: w.monitor,
                        windows: w.windows,
                    })
                    .collect();

                // Sort workspaces by ID for consistent display
                info.sort_by_key(|w| w.id);
                info
            }
            Err(e) => {
                eprintln!("Failed to fetch workspaces: {:?}", e);
                Vec::new()
            }
        };

        let active_id = match Workspace::get_active() {
            Ok(ws) => Some(ws.id),
            Err(e) => {
                eprintln!("Failed to fetch active workspace: {:?}", e);
                None
            }
        };

        (workspaces, active_id)
    }

    /// Switch to a specific workspace.
    async fn switch_workspace(workspace_id: WorkspaceId) {
        let dispatch = DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(workspace_id));

        if let Err(e) = Dispatch::call_async(dispatch).await {
            eprintln!("Failed to switch to workspace {}: {:?}", workspace_id, e);
        }
    }

    /// Create workspace button widgets.
    fn create_workspace_buttons(&self) -> Row<'_, Message> {
        let buttons = self.workspaces.iter().map(|workspace| {
            let is_active = self.active_workspace_id == Some(workspace.id);
            self.create_workspace_button(workspace, is_active)
        });

        Row::from_vec(buttons.collect())
            .spacing(4)
            .align_y(iced::Alignment::Center)
    }

    /// Create a single workspace button.
    fn create_workspace_button<'a>(
        &self,
        workspace: &'a WorkspaceInfo,
        is_active: bool,
    ) -> Element<'a, Message> {
        let label = text(&workspace.id_string).size(13);

        button(label)
            .padding([5, 8])
            .style(move |theme: &iced::Theme, status| {
                Self::workspace_button_style(theme, status, is_active)
            })
            .on_press(Message::WorkspaceClicked(workspace.id))
            .into()
    }

    /// Style function for workspace buttons.
    fn workspace_button_style(
        _theme: &iced::Theme,
        status: button::Status,
        is_active: bool,
    ) -> button::Style {
        // Tokyo Night colors
        let accent = Color::from_rgb8(0x7a, 0xa2, 0xf7);   // Blue accent
        let text_color = Color::from_rgb8(0xc0, 0xca, 0xf5); // Foreground
        let muted = Color::from_rgb8(0x56, 0x5f, 0x89);     // Muted text
        let hover_bg = Color::from_rgba8(0x41, 0x48, 0x68, 0.5); // Hover background

        let (background, border_color, txt) = match (is_active, status) {
            (true, _) => (None, accent, text_color),
            (false, button::Status::Hovered | button::Status::Pressed) => {
                (Some(hover_bg.into()), muted, text_color)
            }
            (false, _) => (None, Color::TRANSPARENT, muted),
        };

        button::Style {
            background,
            text_color: txt,
            border: Border {
                color: border_color,
                width: 2.0,
                radius: 4.0.into(),
            },
            shadow: Default::default(),
        }
    }

    /// Run the Hyprland event listener and send refresh messages on events.
    async fn run_event_listener<S>(output: &mut S)
    where
        S: SinkExt<Message> + Clone + Unpin + Send + Sync + 'static,
        S::Error: std::fmt::Debug,
    {
        let mut listener = AsyncEventListener::new();

        // Helper to create event handlers with less boilerplate
        let create_handler = |output: S| {
            move |_| {
                let mut output = output.clone();
                Box::pin(async move {
                    let _ = output.send(Message::Refresh).await;
                })
                    as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            }
        };

        // Register handlers for all workspace-related events
        listener.add_workspace_added_handler(create_handler(output.clone()));
        listener.add_workspace_deleted_handler(create_handler(output.clone()));
        listener.add_workspace_changed_handler(create_handler(output.clone()));

        // Start listening for events
        if let Err(e) = listener.start_listener_async().await {
            eprintln!("Hyprland event listener error: {:?}", e);
        }
    }
}
