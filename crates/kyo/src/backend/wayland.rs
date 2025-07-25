use std::collections::HashMap;

use smithay_client_toolkit::{
    delegate_registry,
    output::OutputState,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
    shell::{WaylandSurface, wlr_layer::LayerSurface, xdg::window::Window},
};
use tokio::sync::mpsc::UnboundedSender;
use wayland_backend::client::ObjectId;
use wayland_client::{
    Proxy, QueueHandle,
    globals::GlobalList,
    protocol::{wl_keyboard::WlKeyboard, wl_pointer::WlPointer},
};

use crate::prelude::*;

pub(crate) mod compositor;
pub(crate) mod keyboard;
pub(crate) mod layer;
pub(crate) mod output;
pub(crate) mod pointer;
pub(crate) mod seat;
pub(crate) mod window;

pub(crate) enum SurfaceHandle {
    Layer(LayerSurface),
    Window(Window),
}

impl SurfaceHandle {
    pub(crate) fn id(&self) -> ObjectId {
        match self {
            Self::Layer(layer) => layer.wl_surface().id(),
            Self::Window(window) => window.wl_surface().id(),
        }
    }
}

pub(crate) struct WaylandView<Message> {
    pub(crate) id: ObjectId,
    pub(crate) surface: SurfaceHandle,

    pub(crate) widget: View<Message>,
}

impl<Message> WaylandView<Message> {
    pub(crate) fn new(surface: SurfaceHandle, widget: View<Message>) -> Self {
        Self {
            id: surface.id(),
            surface,
            widget,
        }
    }

    pub(crate) fn destroy(&self) {
        match &self.surface {
            SurfaceHandle::Layer(layer) => layer.wl_surface().destroy(),
            SurfaceHandle::Window(window) => window.wl_surface().destroy(),
        }
    }
}

pub(crate) struct State<Message> {
    pub(crate) client: UnboundedSender<Element<Message>>,

    pub(crate) views: HashMap<ObjectId, WaylandView<Message>>,
    pub(crate) lut: HashMap<String, ObjectId>,

    pub(crate) registry_state: RegistryState,
    pub(crate) seat_state: SeatState,
    pub(crate) output_state: OutputState,

    pub(crate) keyboard: Option<WlKeyboard>,
    pub(crate) pointer: Option<WlPointer>,
}

impl<Message: 'static + Send + Sync> State<Message> {
    pub(crate) fn new(
        client: UnboundedSender<Element<Message>>,
        globals: &GlobalList,
        qh: &QueueHandle<Self>,
    ) -> Self {
        Self {
            registry_state: RegistryState::new(globals),
            seat_state: SeatState::new(globals, qh),
            output_state: OutputState::new(globals, qh),

            keyboard: None,
            pointer: None,

            client,
            views: HashMap::new(),
            lut: HashMap::new(),
        }
    }
}

delegate_registry!(@<Message: 'static + Send + Sync> State<Message>);

impl<Message: 'static + Send + Sync> ProvidesRegistryState for State<Message> {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
