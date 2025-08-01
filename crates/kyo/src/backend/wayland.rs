use std::collections::HashMap;

use smithay_client_toolkit::{
    delegate_registry,
    output::OutputState,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
};
use wayland_backend::client::ObjectId;
use wayland_client::{
    QueueHandle,
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

pub(crate) struct State<Message> {
    pub(crate) submitter: Submitter<Message>,
    pub(crate) closer: Submitter<String>,

    pub(crate) views: HashMap<ObjectId, WaylandWidget<Message>>,
    pub(crate) lut: HashMap<String, ObjectId>,

    pub(crate) registry_state: RegistryState,
    pub(crate) seat_state: SeatState,
    pub(crate) output_state: OutputState,

    pub(crate) keyboard: Option<WlKeyboard>,
    pub(crate) pointer: Option<WlPointer>,
}

impl<Message: 'static + Send + Sync> State<Message> {
    pub(crate) fn new(
        submitter: Submitter<Message>,
        closer: Submitter<String>,
        globals: &GlobalList,
        qh: &QueueHandle<Self>,
    ) -> Self {
        Self {
            registry_state: RegistryState::new(globals),
            seat_state: SeatState::new(globals, qh),
            output_state: OutputState::new(globals, qh),

            keyboard: None,
            pointer: None,

            submitter,
            closer,

            views: HashMap::new(),
            lut: HashMap::new(),
        }
    }

    pub(crate) fn throw_event(&mut self, id: Option<ObjectId>, event: Event) {
        if let Some(id) = id {
            if let Some(view) = self.views.get_mut(&id) {
                if let Some(label) = view.on_event(event.clone(), self.submitter.clone()) {
                    self.closer.submit(label).unwrap_or_else(|e| {
                        tracing::error!("Failed to submit a close request for this label: {}", e);
                    });
                }
            }
        } else {
            for view in self.views.values_mut() {
                if let Some(label) = view.on_event(event.clone(), self.submitter.clone()) {
                    self.closer.submit(label).unwrap_or_else(|e| {
                        tracing::error!("Failed to submit a close request for this label: {}", e);
                    });
                }
            }
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
