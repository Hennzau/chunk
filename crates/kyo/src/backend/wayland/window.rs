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
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &Window) {}

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
    }
}
