use std::{any::Any, pin::Pin};

use smithay_client_toolkit::shell::{
    wlr_layer::{Anchor, KeyboardInteractivity, Layer},
    xdg::window::WindowDecorations,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::prelude::*;

pub struct View<Message> {
    pub(crate) window: bool,

    pub(crate) layer: Layer,
    pub(crate) label: String,
    pub(crate) anchor: Anchor,
    pub(crate) keyboard_interactivity: KeyboardInteractivity,
    pub(crate) exclusive_zone: i32,
    pub(crate) size: (u32, u32),
    pub(crate) margin: (i32, i32, i32, i32),
    pub(crate) min_size: Option<(u32, u32)>,
    pub(crate) max_size: Option<(u32, u32)>,
    pub(crate) decorations: WindowDecorations,

    pub(crate) child: Option<Element<Message>>,
}

impl<Message> Default for View<Message> {
    fn default() -> Self {
        View {
            window: false,
            layer: Layer::Top,
            label: String::new(),
            anchor: Anchor::TOP,
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            exclusive_zone: 24,
            size: (1920, 24),
            margin: (0, 0, 0, 0),
            min_size: None,
            max_size: None,
            decorations: WindowDecorations::ServerDefault,
            child: None,
        }
    }
}

impl<Message: 'static + Send + Sync> Widget<Message> for View<Message> {
    fn on_event<'a>(
        &'a mut self,
        event: &'a Event,
        client: &'a UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    where
        Self: Sync + 'a,
    {
        Box::pin(async move {
            if let Some(child) = &mut self.child {
                child.on_event(event, client).await?;
            }

            Ok(())
        })
    }

    fn render(&self, renderer: &mut Renderer) -> Result<()> {
        if let Some(child) = &self.child {
            child.render(renderer)?;
        }

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

impl<Message> View<Message> {
    /// Sets the label for the view.
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the size of the view.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = (width, height);
        self
    }

    /// Sets the anchor for the view.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the exclusive zone for the view.
    pub fn exclusive_zone(mut self, zone: i32) -> Self {
        self.exclusive_zone = zone;
        self
    }

    /// Sets the margin for the view.
    pub fn margin(mut self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
        self.margin = (top, right, bottom, left);
        self
    }

    /// Sets whether this view should be a window.
    pub fn window(mut self, window: bool) -> Self {
        self.window = window;
        self
    }

    /// Sets the decorations for the view.
    pub fn decorations(mut self, decorations: WindowDecorations) -> Self {
        self.decorations = decorations;
        self
    }

    /// Sets the child element of this view.
    pub fn child<E: Into<Element<Message>>>(mut self, child: E) -> Self {
        self.child = Some(child.into());
        self
    }

    /// Sets the minimum size for the view.
    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.min_size = Some((width, height));
        self
    }

    /// Sets the maximum size for the view.
    pub fn max_size(mut self, width: u32, height: u32) -> Self {
        self.max_size = Some((width, height));
        self
    }

    /// Set the layer for the view.
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    /// Set the keyboard interactivity for the view.
    pub fn keyboard_interactivity(mut self, keyboard_interactivity: KeyboardInteractivity) -> Self {
        self.keyboard_interactivity = keyboard_interactivity;
        self
    }

    /// Sets the anchor for the view.
    pub fn anchor_mut(&mut self, anchor: Anchor) -> &mut Self {
        self.anchor = anchor;
        self
    }
}

pub fn view<Message>() -> View<Message> {
    View::default()
}

pub struct Views<Message> {
    pub(crate) views: Vec<View<Message>>,
}

impl<Message> Views<Message> {
    pub fn view(mut self, view: View<Message>) -> Self {
        self.views.push(view);
        self
    }
}

impl<Message: 'static> Widget<Message> for Views<Message> {
    fn on_event<'a>(
        &'a mut self,
        _event: &'a Event,
        _client: &'a UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    where
        Self: Sync + 'a,
    {
        // This widget will never be used as a widget itself, so we can return an empty future.
        Box::pin(async move { Ok(()) })
    }

    fn render(&self, _renderer: &mut Renderer) -> Result<()> {
        // This widget will never be used as a widget itself, so we can return Ok(()).
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
pub fn views<Message>() -> Views<Message> {
    Views { views: Vec::new() }
}
