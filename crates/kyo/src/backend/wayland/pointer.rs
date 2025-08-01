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
        for event in events {
            let (x, y) = event.position;

            self.throw_event(
                Some(event.surface.id()),
                match event.kind {
                    PointerEventKind::Enter { serial: _ } => Event::PointerEntered,
                    PointerEventKind::Leave { serial: _ } => Event::PointerLeaved,
                    PointerEventKind::Motion { time: _ } => Event::PointerMoved { x, y },
                    PointerEventKind::Press {
                        time: _,
                        button,
                        serial: _,
                    } => Event::PointerPressed { x, y, button },
                    PointerEventKind::Release {
                        time: _,
                        button,
                        serial: _,
                    } => Event::PointerReleased { x, y, button },
                    PointerEventKind::Axis {
                        time: _,
                        horizontal,
                        vertical,
                        source: _,
                    } => Event::PointerScrolled {
                        x,
                        y,
                        delta_x: horizontal.absolute,
                        delta_y: vertical.absolute,
                    },
                },
            );
        }
    }
}
