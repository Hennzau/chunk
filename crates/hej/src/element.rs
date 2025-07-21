//! Element module for the GUI framework.

use std::{any::Any, pin::Pin};

use eyre::OptionExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::prelude::*;

/// A struct representing a GUI element that can handle events and render itself.
pub struct Element<Message> {
    pub(crate) widget: Box<dyn Widget<Message>>,
}

impl<Message: 'static> Element<Message> {
    /// Creates an empty element, suitable for use as a placeholder.
    pub fn empty() -> Self {
        EmptyWidget {}.element()
    }

    /// This function is called when an event occurs on the widget.
    /// The widget can then send messages to the application based on the event.
    pub async fn on_event(
        &mut self,
        event: &Event,
        client: &UnboundedSender<Message>,
    ) -> Result<()> {
        self.widget.on_event(event, client).await
    }

    /// This function is called to render the widget using the provided renderer.
    pub fn render(&self, renderer: &mut Renderer) -> Result<()> {
        self.widget.render(renderer)
    }

    /// This function returns a reference to the widget as a trait object.
    pub fn downcast_ref<'a, T: Widget<Message>>(&'a self) -> Result<&'a T> {
        self.widget
            .as_any()
            .downcast_ref::<T>()
            .ok_or_eyre("Failed to downcast Element")
    }

    /// This function returns a mutable reference to the widget as a trait object.
    pub fn downcast_mut<'a, T: Widget<Message>>(&'a mut self) -> Result<&'a mut T> {
        self.widget
            .as_any_mut()
            .downcast_mut::<T>()
            .ok_or_eyre("Failed to downcast Element")
    }

    /// This function consumes the element and returns the underlying widget as a trait object.
    pub fn downcast<T: Widget<Message>>(self) -> Result<Box<T>, Box<dyn Any>> {
        self.widget.into_any().downcast::<T>()
    }
}

/// A trait that implements the conversion of a widget into an element.
pub trait IntoElement<Message> {
    fn element(self) -> Element<Message>;
}

impl<Message, T> IntoElement<Message> for T
where
    T: Widget<Message> + 'static,
{
    /// Converts the widget into an `Element`.
    fn element(self) -> Element<Message> {
        Element {
            widget: Box::new(self),
        }
    }
}

/// A widget that does not render anything and does not handle any events.
pub struct EmptyWidget {}

impl<Message> Widget<Message> for EmptyWidget {
    /// Handles no events and does nothing.
    fn on_event<'a>(
        &'a mut self,
        _event: &'a Event,
        _client: &'a UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    where
        Self: Sync + 'a,
    {
        Box::pin(async move { Ok(()) })
    }

    /// Renders nothing.
    fn render(&self, _renderer: &mut Renderer) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Makes an 'EmptyWidget'.
pub fn empty() -> EmptyWidget {
    EmptyWidget {}
}
