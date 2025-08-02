use std::{pin::Pin, time::Duration};

use crate::prelude::*;

pub(crate) mod wayland;
use eyre::OptionExt;
pub(crate) use wayland::*;

use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface},
        xdg::{
            XdgShell,
            window::{Window, WindowDecorations},
        },
    },
};

use wayland_client::{Connection, EventQueue, globals::registry_queue_init};
use wgpu::{Adapter, Device, Instance, PowerPreference, Queue, RequestAdapterOptions};

pub struct WaylandBackend<Message> {
    pub(crate) submitter: Submitter<Element<Message>>,
    pub(crate) server: Server<Element<Message>>,

    pub(crate) closer: Submitter<String>,
    pub(crate) closer_server: Server<String>,

    // It is important to first destroy state, then the wgpu primitives, then the wayland primitives
    // At some point I should move to a ManuallyDrop struct
    pub(crate) state: State<Message>,

    pub(crate) instance: Instance,
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,

    pub(crate) event_queue: EventQueue<State<Message>>,
    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell: XdgShell,
    pub(crate) layer_shell: LayerShell,
    pub(crate) connection: Connection,
}

impl<Message: 'static + Send + Sync> Backend<Message> for WaylandBackend<Message> {
    async fn new(msg_submitter: Submitter<Message>) -> Result<Self> {
        let (submitter, server) = channel();
        let (closer, closer_server) = channel();

        let connection = Connection::connect_to_env()?;

        let (globals, event_queue) = registry_queue_init::<State<Message>>(&connection)?;
        let qh = event_queue.handle();

        let compositor_state = CompositorState::bind(&globals, &qh)?;
        let xdg_shell = XdgShell::bind(&globals, &qh)?;
        let layer_shell = LayerShell::bind(&globals, &qh)?;

        let instance = Instance::default();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::LowPower,
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter.request_device(&Default::default()).await?;

        let state = State::new(msg_submitter, closer.clone(), &globals, &qh);

        Ok(Self {
            connection,
            event_queue,
            compositor_state,
            xdg_shell,
            layer_shell,

            submitter,
            server,
            closer,
            closer_server,

            instance,
            adapter,
            device,
            queue,

            state,
        })
    }

    fn submitter(&self) -> Submitter<Element<Message>> {
        self.submitter.clone()
    }

    fn closer(&self) -> Submitter<String> {
        self.closer.clone()
    }

    fn run(mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            tracing::info!("Wayland backend started");

            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(16)) => {
                        self.event_queue.flush()?;

                        if let Some(guard) = self.event_queue.prepare_read() {
                            if let Err(e) = guard.read_without_dispatch() {
                                eprintln!("Error reading events: {:?}", e);
                            }
                        }

                        self.event_queue.dispatch_pending(&mut self.state).unwrap();
                    }
                    Ok(element) = self.server.recv() => {
                        let mut lut = self.state.lut.clone();

                        for element in element.into_list() {
                            if element.label().is_none() {
                                tracing::warn!("You submitted a widget with no label, which is forbidden.");

                                continue;
                            }

                            let label = element.label().unwrap();

                            if !lut.contains_key(&label) {
                                let widget =
                                    WaylandWidget::new(self.create_wayland_surface(&element)?, element);

                                lut.insert(label.clone(), widget.id.clone());

                                self.state
                                    .lut
                                    .insert(label.clone(), widget.id.clone());

                                self.state
                                    .views
                                    .insert(widget.id.clone(), widget);
                            }
                        }

                    },
                    Ok(label) = self.closer_server.recv() => {
                        if let Some(id) = self.state.lut.remove(&label) {
                            if let Some(widget) = self.state.views.remove(&id) {
                                widget.destroy();
                            }
                        }
                    }
                }
            }
        })
    }
}

impl<Message: 'static + Send + Sync> WaylandBackend<Message> {
    pub(crate) fn create_layer(
        &self,
        layer: Layer,
        label: String,
        anchor: Anchor,
        keyboard_interactivity: KeyboardInteractivity,
        size: (u32, u32),
        exclusive_zone: u32,
        margin: (i32, i32, i32, i32),
    ) -> LayerSurface {
        let wl_surface = self
            .compositor_state
            .create_surface(&self.event_queue.handle());

        let layer = self.layer_shell.create_layer_surface(
            &self.event_queue.handle(),
            wl_surface,
            layer,
            Some(label.clone()),
            None,
        );

        layer.set_anchor(anchor);
        layer.set_keyboard_interactivity(keyboard_interactivity);
        layer.set_size(size.0, size.1);
        layer.set_exclusive_zone(exclusive_zone as i32);
        layer.set_margin(margin.0, margin.1, margin.2, margin.3);

        layer.commit();

        layer
    }

    pub(crate) fn create_window(
        &self,
        decorations: WindowDecorations,
        label: String,
        min_size: Option<(u32, u32)>,
        max_size: Option<(u32, u32)>,
    ) -> Window {
        let wl_surface = self
            .compositor_state
            .create_surface(&self.event_queue.handle());

        let window =
            self.xdg_shell
                .create_window(wl_surface, decorations, &self.event_queue.handle());

        window.set_title(&label);
        window.set_app_id(&label);
        window.set_min_size(min_size);
        window.set_max_size(max_size);

        window.commit();

        window
    }

    pub(crate) fn create_wayland_surface(
        &self,
        element: &Element<Message>,
    ) -> Result<SurfaceHandle> {
        let (anchor, exclusive) = match element.layout().placement {
            Placement::Top => (Anchor::TOP, element.layout().height),
            Placement::Bottom => (Anchor::BOTTOM, element.layout().height),
            Placement::Left => (Anchor::LEFT, element.layout().width),
            Placement::Right => (Anchor::RIGHT, element.layout().width),
            Placement::Windowed => {
                let window = self.create_window(
                    WindowDecorations::ServerDefault,
                    element.label().ok_or_eyre(
                        "Element must have a label in order to build a wayland layer",
                    )?,
                    None,
                    None,
                );

                return Ok(SurfaceHandle::from_window(
                    window,
                    self.instance.clone(),
                    self.connection.clone(),
                    self.adapter.clone(),
                    self.device.clone(),
                    self.queue.clone(),
                ));
            }
            Placement::None => (Anchor::TOP | Anchor::LEFT, 0),
        };

        let layer = self.create_layer(
            Layer::Top,
            element
                .label()
                .ok_or_eyre("Element must have a label in order to build a wayland layer")?,
            anchor,
            match element.layout().keyboard_sensitivity {
                KeyboardSensitivity::None => KeyboardInteractivity::None,
                KeyboardSensitivity::OnClick => KeyboardInteractivity::OnDemand,
                KeyboardSensitivity::Exclusive => KeyboardInteractivity::Exclusive,
            },
            (element.layout().width, element.layout().height),
            exclusive,
            (element.layout().y as i32, 0, 0, element.layout().x as i32),
        );

        Ok(SurfaceHandle::from_layer(
            layer,
            self.instance.clone(),
            self.connection.clone(),
            self.adapter.clone(),
            self.device.clone(),
            self.queue.clone(),
        ))
    }
}
