//! This module defines the `Widget` trait, which is used to create interactive UI components.
//! Widgets can handle events and render themselves using a `Renderer`.

use std::pin::Pin;

use tokio::sync::mpsc::UnboundedSender;

use crate::prelude::*;

/// The `Widget` trait defines the interface for interactive UI components.
///
/// Example implementation:
/// ```rust
/// use hej::prelude::{reexport::*, *};
///
/// use std::pin::Pin;
/// use tokio::sync::mpsc::UnboundedSender;
///
/// enum Message {}
///
/// struct MyWidget;
///
/// impl Widget<Message> for MyWidget {
///     fn on_event<'a>(
///         &'a mut self,
///         _event: &'a Event,
///         _client: &'a UnboundedSender<Message>,
///     ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
///     where
///         Self: Sync + 'a,{
///         Box::pin(async { Ok(()) })
///     }
///
///     fn render(&self, _renderer: &mut Renderer) -> Result<()> {
///         Ok(())
///     }
/// }
/// ```
pub trait Widget<Message>: Send + Sync {
    /// This function is called when an event occurs on the widget.
    /// The widget can then send messages to the application based on the event.
    fn on_event<'a>(
        &'a mut self,
        event: &'a Event,
        client: &'a UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    where
        Self: Sync + 'a;

    /// This function is called to render the widget using the provided renderer.
    fn render(&self, renderer: &mut Renderer) -> Result<()>;
}
