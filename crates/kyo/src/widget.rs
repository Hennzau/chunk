use wayland_backend::client::ObjectId;

use crate::prelude::*;

pub(crate) struct WaylandWidget<Message> {
    pub(crate) id: ObjectId,
    pub(crate) surface: SurfaceHandle,

    pub(crate) widget: Element<Message>,
}

impl<Message: 'static + Send + Sync> WaylandWidget<Message> {
    pub(crate) fn new(surface: SurfaceHandle, widget: Element<Message>) -> Self {
        Self {
            id: surface.id(),
            surface,
            widget,
        }
    }

    pub(crate) fn destroy(&self) {
        self.surface.destroy();
    }

    pub(crate) fn on_event(
        &mut self,
        event: Event,
        submitter: Submitter<Message>,
    ) -> Option<String> {
        match event {
            Event::Close => self.widget.label(),
            Event::Configure { width, height } => {
                self.surface.configure(width, height);

                None
            }
            event => {
                if let Err(e) = self.widget.on_event(event, submitter) {
                    tracing::error!("Error {}", e);
                }

                None
            }
        }
    }
}
