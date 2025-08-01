//! Element module for the GUI framework.

use eyre::OptionExt;

use crate::prelude::*;

/// A struct representing a GUI element that can handle events and render itself.
pub struct Element<Message> {
    pub(crate) widget: Box<dyn Widget<Message>>,
}

impl<Message: 'static + Send + Sync> Element<Message> {
    /// Creates an empty element, suitable for use as a placeholder.
    pub fn empty() -> Self {
        empty().element()
    }

    pub fn layout(&self) -> Layout {
        self.widget.layout()
    }

    pub fn label(&self) -> Option<String> {
        self.widget.label()
    }

    /// This function is called when an event occurs on the widget.
    /// The widget can then send messages to the application based on the event.
    pub fn on_event(&mut self, event: Event, client: Submitter<Message>) -> Result<()> {
        self.widget.on_event(event, client)
    }

    /// This function is called to render the widget using the provided renderer.
    pub fn draw(&self, canvas: Canvas, renderer: &mut Renderer) -> Result<()> {
        self.widget.draw(canvas, renderer)
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
    pub fn downcast<T: Widget<Message>>(self) -> Result<Box<T>, Self> {
        if self.widget.as_any().downcast_ref::<T>().is_some() {
            return Ok(self
                .widget
                .into_any()
                .downcast::<T>()
                .expect("Downcasting should have worked..."));
        }

        Err(self)
    }

    /// Maps this Element<Message> to another Element<NewMessage>
    pub fn map<NewMessage: 'static + Send + Sync>(
        self,
        map: Map<Message, NewMessage>,
    ) -> Element<NewMessage> {
        Element {
            widget: Box::new(MapWidget::new(self.widget, map)),
        }
    }

    pub fn into_list(self) -> Vec<Element<Message>> {
        match self.downcast::<ContainerWidget<Message>>() {
            Ok(container) => container.elements,
            Err(element) => vec![element],
        }
    }

    pub fn labels(&self) -> Vec<Option<String>> {
        match self.downcast_ref::<ContainerWidget<Message>>() {
            Ok(container) => container.elements.iter().map(|e| e.label()).collect(),
            Err(_) => vec![self.label()],
        }
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
