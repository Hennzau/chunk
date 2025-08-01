use smithay_client_toolkit::{
    delegate_xdg_shell, delegate_xdg_window,
    shell::{
        WaylandSurface,
        xdg::window::{Window, WindowConfigure, WindowHandler},
    },
};
use wayland_client::{Connection, Proxy, QueueHandle};

use crate::prelude::*;

delegate_xdg_shell!(@<Message: 'static + Send + Sync> State<Message>);
delegate_xdg_window!(@<Message: 'static + Send + Sync> State<Message>);

impl<Message: 'static + Send + Sync> WindowHandler for State<Message> {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &Window) {
        self.throw_event(Some(window.wl_surface().id()), Event::Close);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        self.throw_event(
            Some(window.wl_surface().id()),
            Event::Configure {
                width: configure.new_size.0.map(|n| n.get()).unwrap_or(0),
                height: configure.new_size.1.map(|n| n.get()).unwrap_or(0),
            },
        );
    }
}
