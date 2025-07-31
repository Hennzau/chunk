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
        xdg::XdgShell,
    },
};

use wayland_client::{
    Connection, EventQueue,
    globals::{GlobalList, registry_queue_init},
};

pub struct WaylandBackend<Message> {
    pub(crate) submitter: Submitter<Element<Message>>,
    pub(crate) server: Server<Element<Message>>,

    pub(crate) closer: Submitter<String>,
    pub(crate) closer_server: Server<String>,

    pub(crate) globals: GlobalList,
    pub(crate) event_queue: EventQueue<State<Message>>,

    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell: XdgShell,
    pub(crate) layer_shell: LayerShell,
}

impl<Message: 'static + Send + Sync> Backend<Message> for WaylandBackend<Message> {
    async fn new() -> Result<Self> {
        let (submitter, server) = channel();
        let (closer, closer_server) = channel();

        let connection = Connection::connect_to_env()?;

        let (globals, event_queue) = registry_queue_init::<State<Message>>(&connection)?;
        let qh = event_queue.handle();

        let compositor_state = CompositorState::bind(&globals, &qh)?;
        let xdg_shell = XdgShell::bind(&globals, &qh)?;
        let layer_shell = LayerShell::bind(&globals, &qh)?;

        Ok(Self {
            globals,
            event_queue,
            compositor_state,
            xdg_shell,
            layer_shell,

            submitter,
            server,
            closer,
            closer_server,
        })
    }

    fn submitter(&self) -> Submitter<Element<Message>> {
        self.submitter.clone()
    }

    fn closer(&self) -> Submitter<String> {
        self.closer.clone()
    }

    fn run(
        mut self,
        msg_submitter: Submitter<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            tracing::info!("Wayland backend started");

            let mut state = State::new(msg_submitter, &self.globals, &self.event_queue.handle());

            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(16)) => {
                        self.event_queue.flush()?;

                        if let Some(guard) = self.event_queue.prepare_read() {
                            if let Err(e) = guard.read_without_dispatch() {
                                eprintln!("Error reading events: {:?}", e);
                            }
                        }

                        self.event_queue.dispatch_pending(&mut state).unwrap();
                    }
                    Ok(element) = self.server.recv() => {
                        let mut lut = state.lut.clone();
                        let mut new_labels = Vec::new();

                        match element.downcast::<ContainerWidget<Message>>() {
                            Ok(container) => {
                                tracing::debug!("Received a container");

                                for element in container.elements() {
                                    if element.label().is_none() {
                                        continue;
                                    }

                                    let label = element.label().clone().unwrap(); // safe

                                    if !lut.contains_key(&label) {
                                        tracing::info!("New element received: {}", label);

                                        let widget =
                                            WaylandWidget::new(self.create_wayland_surface(&element)?, element);

                                        new_labels.push(label.clone());

                                        lut.insert(label.clone(), widget.id.clone());

                                        state
                                            .lut
                                            .insert(label.clone(), widget.id.clone());

                                        state
                                            .views
                                            .insert(widget.id.clone(), widget);
                                    } else {
                                        tracing::debug!("View already exists: {}", label);
                                    }
                                }
                            }
                            Err(element) => {
                                tracing::debug!("Received a single element");

                                if element.label().is_none() {
                                    continue;
                                }

                                let label = element.label().clone().unwrap(); // safe

                                if !lut.contains_key(&label) {
                                    tracing::info!("New element received: {}", label);

                                    let widget =
                                        WaylandWidget::new(self.create_wayland_surface(&element)?, element);

                                    new_labels.push(label.clone());

                                    lut.insert(label.clone(), widget.id.clone());

                                    state
                                        .lut
                                        .insert(label.clone(), widget.id.clone());

                                    state
                                        .views
                                        .insert(widget.id.clone(), widget);
                                } else {
                                    tracing::debug!("View already exists: {}", label);
                                }
                            }
                        }

                        for (label, id) in lut {
                            if !new_labels.contains(&label) {
                                tracing::debug!("Removing view: {}", label);

                                if let Some(view) = state.views.remove(&id) {
                                    view.destroy();
                                }

                                state.lut.remove(&label);
                            }
                        }

                    },
                    Ok(label) = self.closer_server.recv() => {
                        if let Some(id) = state.lut.remove(&label) {
                            if let Some(widget) = state.views.remove(&id) {
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

    pub(crate) fn create_wayland_surface(
        &self,
        element: &Element<Message>,
    ) -> Result<SurfaceHandle> {
        let (anchor, exclusive, margin) = match element.layout().reserve {
            Some(reserve) => match reserve {
                Reserve::Top => (Anchor::TOP, element.layout().height, (0, 0, 0, 0)),
                Reserve::Bottom => (Anchor::BOTTOM, element.layout().height, (0, 0, 0, 0)),
                Reserve::Left => (Anchor::LEFT, element.layout().width, (0, 0, 0, 0)),
                Reserve::Right => (Anchor::RIGHT, element.layout().width, (0, 0, 0, 0)),
            },
            None => (
                Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                0,
                (0, 0, 0, 0),
            ),
        };

        let layer = self.create_layer(
            Layer::Top,
            element
                .label()
                .ok_or_eyre("Element must have a label in order to build a wayland layer")?,
            anchor,
            KeyboardInteractivity::OnDemand,
            (element.layout().width, element.layout().height),
            exclusive,
            margin,
        );

        Ok(SurfaceHandle::Layer(layer))
    }
}
