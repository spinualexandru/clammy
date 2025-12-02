//! Workspaces component for displaying and managing Hyprland workspaces.
//!
//! This component provides:
//! - Real-time workspace list display
//! - Active workspace highlighting
//! - Click-to-switch functionality
//! - Automatic updates via Hyprland event subscription

use hyprland::data::{Workspace, Workspaces as HyprWorkspaces};
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::shared::{HyprData, HyprDataActive, WorkspaceId};
use iced::widget::{Row, button, container, row, stack, text};
use iced::{Border, Element, Length, Subscription, Task};

use crate::hyprland_events::HyprlandSubscription;
use crate::theme::get_theme;

// ============================================================================
// Constants - Fine-tune these for perfect alignment
// ============================================================================

/// Button padding (vertical, horizontal)
const BUTTON_PADDING_V: f32 = 5.0;
const BUTTON_PADDING_H: f32 = 8.0;

/// Text size for workspace labels
const TEXT_SIZE: f32 = 13.0;

/// Approximate text width for single-digit workspace IDs
const TEXT_WIDTH_APPROX: f32 = 8.0;

/// Total width of each workspace button (text + horizontal padding)
const BUTTON_WIDTH: f32 = TEXT_WIDTH_APPROX + (BUTTON_PADDING_H * 2.0);

/// Spacing between workspace buttons
const BUTTON_SPACING: f32 = 4.0;

/// Row padding (horizontal)
const ROW_PADDING: f32 = 3.0;

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
    /// ID of the previous workspace (for animation)
    previous_workspace_id: Option<WorkspaceId>,
    /// Animation progress (0.0 = old workspace, 1.0 = new workspace)
    animation_progress: f32,
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
    /// Animation tick for border transition
    #[doc(hidden)]
    AnimationTick,
}

// ============================================================================
// Implementation
// ============================================================================

impl Default for Workspaces {
    fn default() -> Self {
        Self {
            workspaces: Vec::new(),
            active_workspace_id: None,
            previous_workspace_id: None,
            animation_progress: 1.0, // Start fully transitioned
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

                // Check if workspace changed to start animation
                if active_id != self.active_workspace_id {
                    self.previous_workspace_id = self.active_workspace_id;
                    self.active_workspace_id = active_id;
                    self.animation_progress = 0.0; // Start animation
                } else {
                    self.active_workspace_id = active_id;
                }

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

            Message::AnimationTick => {
                if self.animation_progress < 1.0 {
                    // Increment animation progress (smooth out over ~200ms at 60fps)
                    self.animation_progress = (self.animation_progress + 0.15).min(1.0);

                    // Clear previous workspace when animation completes
                    if self.animation_progress >= 1.0 {
                        self.previous_workspace_id = None;
                    }
                }
                Task::none()
            }
        }
    }

    /// Render the workspaces component.
    pub fn view(&self) -> Element<'_, Message> {
        let workspace_buttons = self.create_workspace_buttons();

        let buttons_content = workspace_buttons
            .spacing(BUTTON_SPACING as u16)
            .padding([0, ROW_PADDING as u16])
            .align_y(iced::Alignment::Center);

        // Create moving indicator overlay
        let indicator = self.create_moving_indicator();

        // Stack indicator on top of buttons
        let stacked = stack![buttons_content, indicator];

        container(stacked)
            .width(Length::Shrink)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    /// Subscribe to Hyprland workspace events.
    pub fn subscription(&self) -> Subscription<Message> {
        let event_subscription = HyprlandSubscription::new("hyprland-workspace-events")
            .on_any_workspace_event(|| Message::Refresh)
            .build();

        // Add animation subscription when transition is in progress
        let animation_subscription = if self.animation_progress < 1.0 {
            iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| Message::AnimationTick)
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![event_subscription, animation_subscription])
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
            let is_previous = self.previous_workspace_id == Some(workspace.id);
            self.create_workspace_button(workspace, is_active, is_previous)
        });

        Row::from_vec(buttons.collect())
            .spacing(BUTTON_SPACING as u16)
            .align_y(iced::Alignment::Center)
    }

    /// Create a single workspace button.
    fn create_workspace_button<'a>(
        &self,
        workspace: &'a WorkspaceInfo,
        is_active: bool,
        is_previous: bool,
    ) -> Element<'a, Message> {
        let label = text(&workspace.id_string).size(TEXT_SIZE);
        let animation_progress = self.animation_progress;

        button(label)
            .padding([BUTTON_PADDING_V as u16, BUTTON_PADDING_H as u16])
            .style(move |theme: &iced::Theme, status| {
                Self::workspace_button_style(theme, status, is_active, is_previous, animation_progress)
            })
            .on_press(Message::WorkspaceClicked(workspace.id))
            .into()
    }

    /// Style function for workspace buttons.
    fn workspace_button_style(
        _theme: &iced::Theme,
        status: button::Status,
        is_active: bool,
        _is_previous: bool,
        _animation_progress: f32,
    ) -> button::Style {
        let theme = get_theme();
        let text_color = theme.text();
        let muted = theme.muted();
        let hover_bg = theme.hover();

        // No borders on buttons - only hover effect and text color change
        let (background, txt) = if is_active {
            (None, text_color)
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => {
                    (Some(hover_bg.into()), text_color)
                }
                _ => (None, muted),
            }
        };

        button::Style {
            background,
            text_color: txt,
            border: Border::default(), // No border
            shadow: Default::default(),
        }
    }

    /// Find the index of a workspace by its ID in the sorted workspace list.
    fn find_workspace_index(&self, workspace_id: WorkspaceId) -> usize {
        self.workspaces
            .iter()
            .position(|w| w.id == workspace_id)
            .unwrap_or(0)
    }

    /// Create the moving border indicator overlay.
    fn create_moving_indicator(&self) -> Element<'_, Message> {
        use iced::widget::{horizontal_space, Space};

        if let Some(active_id) = self.active_workspace_id {
            let theme = get_theme();
            let accent = theme.accent();

            let active_index = self.find_workspace_index(active_id);
            let prev_index = self
                .previous_workspace_id
                .map(|id| self.find_workspace_index(id))
                .unwrap_or(active_index);

            // Interpolate position between old and new workspace
            let interpolated_pos =
                prev_index as f32 + (active_index as f32 - prev_index as f32) * self.animation_progress;

            // Calculate horizontal offset using constants
            let offset = ROW_PADDING + interpolated_pos * (BUTTON_WIDTH + BUTTON_SPACING);

            // Create indicator with dimensions matching the button exactly
            let indicator_box = container(Space::new(
                Length::Fixed(TEXT_WIDTH_APPROX),
                Length::Fixed(TEXT_SIZE),
            ))
            .padding([BUTTON_PADDING_V as u16, BUTTON_PADDING_H as u16])
            .style(move |_theme| container::Style {
                background: None,
                border: Border {
                    color: accent,
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

            // Use horizontal space to position the indicator, with vertical centering
            row![horizontal_space().width(Length::Fixed(offset)), indicator_box]
                .height(Length::Fill)
                .align_y(iced::Alignment::Center)
                .into()
        } else {
            // No active workspace, return empty space
            Space::new(0, 0).into()
        }
    }

}
