//! This module defines the `Widget` trait, which is used to create interactive UI components.
//! Widgets can handle events and render themselves using a `Renderer`, but they must resolve
//! quickly and return a `Result<()>`.
//! The trait also provides methods for type conversion to `Any`, allowing for dynamic type handling.

use std::any::Any;

use crate::prelude::*;

/// A Convenient type around tokio Unbounded Sender
pub type Sender<T> = tokio::sync::mpsc::UnboundedSender<T>;

pub trait Widget<Message>: Send + Sync + Any {
    /// This function is called when an event occurs on the widget. It must resolves
    /// quickly and return a `Result<()>`. A widget can handle events but with no computation,
    /// only a deterministic, immediate change of state.
    #[allow(unused_variables)]
    fn on_event(&mut self, event: Event, client: Submitter<Message>) -> Result<()> {
        Ok(())
    }

    /// This function is called to draw the widget using the provided renderer on the provided canvas.
    #[allow(unused_variables)]
    fn draw(&self, canvas: Canvas, renderer: &mut Renderer) -> Result<()> {
        Ok(())
    }

    fn layout(&self) -> Layout {
        Layout {
            x: 0,
            y: 0,

            width: 0,
            height: 0,

            reserve: None,
        }
    }

    fn label(&self) -> Option<String> {
        None
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

/// A widget that does not render anything and does not handle any events.
#[derive(Default)]
pub struct EmptyWidget {
    pub(crate) layout: Layout,
    pub(crate) label: Option<String>,
}

impl EmptyWidget {
    pub fn layout(self, layout: Layout) -> Self {
        Self { layout, ..self }
    }

    pub fn label(self, label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..self
        }
    }
}

impl<Message> Widget<Message> for EmptyWidget {
    fn layout(&self) -> Layout {
        self.layout.clone()
    }

    fn label(&self) -> Option<String> {
        self.label.clone()
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
    EmptyWidget::default()
}

/// A widget that maps to another widget
pub struct MapWidget<MessageA, MessageB> {
    widget: Box<dyn Widget<MessageA>>,
    map: Map<MessageA, MessageB>,
}

impl<MessageA, MessageB> MapWidget<MessageA, MessageB> {
    pub fn new(widget: Box<dyn Widget<MessageA>>, map: Map<MessageA, MessageB>) -> Self {
        Self { widget, map }
    }
}

impl<MessageA: 'static + Send + Sync, MessageB: 'static + Send + Sync> Widget<MessageB>
    for MapWidget<MessageA, MessageB>
{
    fn layout(&self) -> Layout {
        self.widget.layout()
    }

    fn on_event(&mut self, event: Event, client: Submitter<MessageB>) -> Result<()> {
        let (sender, mut receiver) = channel::<MessageA>();

        self.widget.on_event(event, sender)?;

        while let Ok(message) = receiver.try_recv() {
            let mapped_message = self.map.map(message);

            client.submit(mapped_message).unwrap_or_else(|_| {
                tracing::error!("Failed to send message from MapWidget");
            });
        }

        Ok(())
    }

    fn draw(&self, canvas: Canvas, renderer: &mut Renderer) -> Result<()> {
        self.widget.draw(canvas, renderer)
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

/// A container widget that contains other widgets with nothing else. This widget
/// is just useful as a "Vec of Elements" that will be consume directly after being passed.
pub struct ContainerWidget<Message> {
    pub(crate) elements: Vec<Element<Message>>,
}

impl<Message> ContainerWidget<Message> {
    pub fn with(mut self, element: impl IntoElement<Message>) -> Self {
        self.elements.push(element.element());

        self
    }

    pub fn elements(self) -> Vec<Element<Message>> {
        self.elements
    }
}

impl<Message: 'static> Widget<Message> for ContainerWidget<Message> {
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

/// Makes an 'ContainerWidget'.
pub fn container<Message>() -> ContainerWidget<Message> {
    ContainerWidget {
        elements: Vec::new(),
    }
}
