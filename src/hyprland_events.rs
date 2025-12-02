//! Hyprland event subscription abstraction.
//!
//! Provides a builder pattern for creating Hyprland event subscriptions
//! with less boilerplate than using `AsyncEventListener` directly.

use hyprland::event_listener::AsyncEventListener;
use iced::futures::SinkExt;
use iced::stream;
use iced::Subscription;
use std::future;
use std::pin::Pin;

/// Type alias for the boxed async handler future.
type BoxedFuture = Pin<Box<dyn std::future::Future<Output = ()> + Send>>;

/// Builder for Hyprland event subscriptions.
///
/// # Example
/// ```ignore
/// HyprlandSubscription::new("my-events")
///     .on_workspace_change(|| Message::Refresh)
///     .on_active_window(|data| Message::WindowChanged(data))
///     .build()
/// ```
pub struct HyprlandSubscription<M> {
    id: &'static str,
    workspace_added: Option<Box<dyn Fn() -> M + Send + Sync + 'static>>,
    workspace_deleted: Option<Box<dyn Fn() -> M + Send + Sync + 'static>>,
    workspace_changed: Option<Box<dyn Fn() -> M + Send + Sync + 'static>>,
    active_window: Option<Box<dyn Fn(Option<(String, String)>) -> M + Send + Sync + 'static>>,
}

impl<M> HyprlandSubscription<M>
where
    M: Clone + Send + 'static,
{
    /// Create a new subscription builder with the given ID.
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            workspace_added: None,
            workspace_deleted: None,
            workspace_changed: None,
            active_window: None,
        }
    }

    /// Handle workspace added events.
    pub fn on_workspace_added<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> M + Send + Sync + 'static,
    {
        self.workspace_added = Some(Box::new(handler));
        self
    }

    /// Handle workspace deleted events.
    pub fn on_workspace_deleted<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> M + Send + Sync + 'static,
    {
        self.workspace_deleted = Some(Box::new(handler));
        self
    }

    /// Handle workspace changed events (active workspace changed).
    pub fn on_workspace_changed<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> M + Send + Sync + 'static,
    {
        self.workspace_changed = Some(Box::new(handler));
        self
    }

    /// Handle all workspace events with a single handler.
    /// Convenience method that sets added, deleted, and changed handlers.
    pub fn on_any_workspace_event<F>(self, handler: F) -> Self
    where
        F: Fn() -> M + Clone + Send + Sync + 'static,
    {
        self.on_workspace_added(handler.clone())
            .on_workspace_deleted(handler.clone())
            .on_workspace_changed(handler)
    }

    /// Handle active window changed events.
    /// The handler receives `Some((title, class))` or `None` if no window is focused.
    pub fn on_active_window<F>(mut self, handler: F) -> Self
    where
        F: Fn(Option<(String, String)>) -> M + Send + Sync + 'static,
    {
        self.active_window = Some(Box::new(handler));
        self
    }

    /// Build the subscription.
    pub fn build(self) -> Subscription<M> {
        let id = self.id;

        Subscription::run_with_id(
            id,
            stream::channel(100, move |output| {
                let workspace_added = self.workspace_added;
                let workspace_deleted = self.workspace_deleted;
                let workspace_changed = self.workspace_changed;
                let active_window = self.active_window;

                async move {
                    run_listener(
                        output,
                        workspace_added,
                        workspace_deleted,
                        workspace_changed,
                        active_window,
                    )
                    .await;

                    // Keep subscription alive
                    future::pending::<()>().await;
                }
            }),
        )
    }
}

/// Internal function to run the event listener with configured handlers.
async fn run_listener<M, S>(
    output: S,
    workspace_added: Option<Box<dyn Fn() -> M + Send + Sync + 'static>>,
    workspace_deleted: Option<Box<dyn Fn() -> M + Send + Sync + 'static>>,
    workspace_changed: Option<Box<dyn Fn() -> M + Send + Sync + 'static>>,
    active_window: Option<Box<dyn Fn(Option<(String, String)>) -> M + Send + Sync + 'static>>,
) where
    M: Clone + Send + 'static,
    S: SinkExt<M> + Clone + Unpin + Send + Sync + 'static,
{
    let mut listener = AsyncEventListener::new();

    // Helper to create workspace event handlers
    macro_rules! add_workspace_handler {
        ($listener:expr, $method:ident, $handler:expr, $output:expr) => {
            if let Some(handler) = $handler {
                let handler = std::sync::Arc::new(handler);
                let output = $output.clone();
                $listener.$method(move |_| {
                    let handler = handler.clone();
                    let mut output = output.clone();
                    Box::pin(async move {
                        let msg = handler();
                        let _ = output.send(msg).await;
                    }) as BoxedFuture
                });
            }
        };
    }

    add_workspace_handler!(
        listener,
        add_workspace_added_handler,
        workspace_added,
        output
    );
    add_workspace_handler!(
        listener,
        add_workspace_deleted_handler,
        workspace_deleted,
        output
    );
    add_workspace_handler!(
        listener,
        add_workspace_changed_handler,
        workspace_changed,
        output
    );

    // Active window handler is slightly different - it receives data
    if let Some(handler) = active_window {
        let handler = std::sync::Arc::new(handler);
        let output = output.clone();
        listener.add_active_window_changed_handler(move |data| {
            let handler = handler.clone();
            let mut output = output.clone();
            Box::pin(async move {
                let window_data = data.map(|w| (w.title, w.class));
                let msg = handler(window_data);
                let _ = output.send(msg).await;
            }) as BoxedFuture
        });
    }

    // Start listener
    if let Err(e) = listener.start_listener_async().await {
        eprintln!("Hyprland event listener error: {:?}", e);
    }
}
