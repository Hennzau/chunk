use std::ptr::NonNull;

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::shell::{WaylandSurface, wlr_layer::LayerSurface, xdg::window::Window};
use wayland_backend::client::ObjectId;
use wayland_client::{Connection, Proxy, protocol::wl_surface::WlSurface};
use wgpu::{
    Adapter, Color, CompositeAlphaMode, Device, Instance, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration,
    SurfaceTargetUnsafe, TextureUsages, TextureViewDescriptor,
};

pub(crate) enum SurfaceHandle {
    Layer {
        layer: LayerSurface,
        surface: Surface<'static>,
        adapter: Adapter,
        device: Device,
        queue: Queue,
    },
    Window {
        window: Window,
        surface: Surface<'static>,
        adapter: Adapter,
        device: Device,
        queue: Queue,
    },
}

impl SurfaceHandle {
    pub(crate) fn surface(&self) -> &Surface<'static> {
        match self {
            Self::Layer {
                layer: _,
                surface,
                adapter: _,
                device: _,
                queue: _,
            } => surface,
            Self::Window {
                window: _,
                surface,
                adapter: _,
                device: _,
                queue: _,
            } => surface,
        }
    }

    pub(crate) fn adapter(&self) -> &Adapter {
        match self {
            Self::Layer {
                layer: _,
                surface: _,
                adapter,
                device: _,
                queue: _,
            } => adapter,
            Self::Window {
                window: _,
                surface: _,
                adapter,
                device: _,
                queue: _,
            } => adapter,
        }
    }

    pub(crate) fn device(&self) -> &Device {
        match self {
            Self::Layer {
                layer: _,
                surface: _,
                adapter: _,
                device,
                queue: _,
            } => device,
            Self::Window {
                window: _,
                surface: _,
                adapter: _,
                device,
                queue: _,
            } => device,
        }
    }

    pub(crate) fn queue(&self) -> &Queue {
        match self {
            Self::Layer {
                layer: _,
                surface: _,
                adapter: _,
                device: _,
                queue,
            } => queue,
            Self::Window {
                window: _,
                surface: _,
                adapter: _,
                device: _,
                queue,
            } => queue,
        }
    }

    pub(crate) fn configure(&self, width: u32, height: u32) {
        let cap = self.surface().get_capabilities(self.adapter());

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: cap.formats[0],
            view_formats: vec![cap.formats[0]],
            alpha_mode: CompositeAlphaMode::Auto,
            width,
            height,
            desired_maximum_frame_latency: 2,
            present_mode: PresentMode::Mailbox,
        };

        self.surface().configure(self.device(), &surface_config);

        let surface_texture = self
            .surface()
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");

        let texture_view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device().create_command_encoder(&Default::default());
        {
            let _renderpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLUE),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue().submit(Some(encoder.finish()));
        surface_texture.present();
    }

    pub(crate) fn id(&self) -> ObjectId {
        match self {
            Self::Layer {
                layer,
                surface: _,
                adapter: _,
                device: _,
                queue: _,
            } => layer.wl_surface().id(),
            Self::Window {
                window,
                surface: _,
                adapter: _,
                device: _,
                queue: _,
            } => window.wl_surface().id(),
        }
    }

    pub(crate) fn destroy(&self) {
        match self {
            Self::Layer {
                layer,
                surface: _,
                adapter: _,
                device: _,
                queue: _,
            } => {
                layer.wl_surface().destroy();
            }
            Self::Window {
                window,
                surface: _,
                adapter: _,
                device: _,
                queue: _,
            } => {
                window.wl_surface().destroy();
            }
        }
    }

    fn wgpu_surface(
        surface: &WlSurface,
        instance: Instance,
        connection: Connection,
    ) -> Surface<'static> {
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(connection.backend().display_ptr() as *mut _).unwrap(),
        ));

        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(surface.id().as_ptr() as *mut _).unwrap(),
        ));

        let surface = unsafe {
            instance
                .create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle,
                    raw_window_handle,
                })
                .unwrap()
        };

        surface
    }

    pub(crate) fn from_layer(
        layer: LayerSurface,

        instance: Instance,
        connection: Connection,

        adapter: Adapter,
        device: Device,
        queue: Queue,
    ) -> Self {
        let surface = Self::wgpu_surface(layer.wl_surface(), instance, connection);

        Self::Layer {
            layer,
            surface,
            adapter,
            device,
            queue,
        }
    }

    pub(crate) fn from_window(
        window: Window,

        instance: Instance,
        connection: Connection,

        adapter: Adapter,
        device: Device,
        queue: Queue,
    ) -> Self {
        let surface = Self::wgpu_surface(window.wl_surface(), instance, connection);

        Self::Window {
            window,
            surface,
            adapter,
            device,
            queue,
        }
    }
}
