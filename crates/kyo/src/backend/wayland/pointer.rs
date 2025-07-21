use crate::prelude::*;

use smithay_client_toolkit::{
    delegate_pointer,
    reexports::client::{Connection, QueueHandle, protocol::wl_pointer::WlPointer},
    seat::pointer::{PointerEvent, PointerEventKind, PointerHandler},
};
use wayland_client::Proxy;

delegate_pointer!(@<Message: 'static + Send + Sync> State<Message>);

impl<Message: 'static + Send + Sync> PointerHandler for State<Message> {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &WlPointer,
        events: &[PointerEvent],
    ) {
    }
}
