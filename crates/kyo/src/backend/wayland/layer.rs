use crate::prelude::*;

use smithay_client_toolkit::{
    delegate_layer,
    reexports::client::{Connection, QueueHandle},
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    },
};
use wayland_client::Proxy;

delegate_layer!(@<Message: 'static + Send + Sync> State<Message>);

impl<Message: 'static + Send + Sync> LayerShellHandler for State<Message> {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        self.throw_event(Some(layer.wl_surface().id()), Event::Close);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.throw_event(
            Some(layer.wl_surface().id()),
            Event::Configure {
                width: configure.new_size.0,
                height: configure.new_size.1,
            },
        );
    }
}
