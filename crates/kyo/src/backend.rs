use std::{pin::Pin, time::Duration};

use crate::prelude::*;

pub(crate) mod wayland;
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
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use wayland_client::{Connection, EventQueue, globals::registry_queue_init};

pub struct WaylandBackend<Message> {
    pub(crate) client: UnboundedSender<Element<Message>>,
    pub(crate) server: UnboundedReceiver<Element<Message>>,

    pub(crate) state: State<Message>,

    pub(crate) event_queue: EventQueue<State<Message>>,
    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell: XdgShell,
    pub(crate) layer_shell: LayerShell,
}

impl<Message: 'static + Send + Sync> Backend<Message> for WaylandBackend<Message> {
    async fn new() -> Result<Self> {
        let (client, server) = tokio::sync::mpsc::unbounded_channel::<Element<Message>>();

        let connection = Connection::connect_to_env()?;

        let (globals, event_queue) = registry_queue_init::<State<Message>>(&connection)?;
        let qh = event_queue.handle();

        let compositor_state = CompositorState::bind(&globals, &qh)?;
        let xdg_shell = XdgShell::bind(&globals, &qh)?;
        let layer_shell = LayerShell::bind(&globals, &qh)?;

        let state = State::new(client.clone(), &globals, &qh);

        Ok(Self {
            event_queue,
            compositor_state,
            xdg_shell,
            layer_shell,
            state,
            client,
            server,
        })
    }

    fn client(&self) -> UnboundedSender<Element<Message>> {
        self.client.clone()
    }

    fn run(
        mut self,
        _client: UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
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
                    Some(element) = self.server.recv() => {
                        let mut lut = self.state.lut.clone();
                        let mut new_labels = Vec::new();

                        match element.downcast::<View<Message>>() {
                            Ok(view) => {
                                tracing::debug!("Received view");

                                if !lut.contains_key(&view.label) {
                                    tracing::info!("New view received: {}", view.label);

                                    let label = view.label.clone();
                                    let wayland_view = WaylandView::new(self.create_wayland_surface(&view), *view);

                                    new_labels.push(label.clone());

                                    lut.insert(label.clone(), wayland_view.id.clone());

                                    self.state
                                        .lut
                                        .insert(label.clone(), wayland_view.id.clone());

                                    self.state
                                        .views
                                        .insert(wayland_view.id.clone(), wayland_view);
                                } else {
                                    tracing::debug!("View already exists: {}", view.label);
                                }
                            }
                            Err(element) => {
                                tracing::debug!("Failed to downcast element to view");

                                match element.downcast::<Views<Message>>() {
                                    Ok(views) => {
                                        tracing::debug!("Received views");
                                        for view in views.views {
                                            if !lut.contains_key(&view.label) {
                                                tracing::info!("New view received: {}", view.label);

                                                let label = view.label.clone();
                                                let wayland_view =
                                                    WaylandView::new(self.create_wayland_surface(&view), view);

                                                new_labels.push(label.clone());

                                                lut.insert(label.clone(), wayland_view.id.clone());

                                                self.state
                                                    .lut
                                                    .insert(label.clone(), wayland_view.id.clone());

                                                self.state
                                                    .views
                                                    .insert(wayland_view.id.clone(), wayland_view);
                                            } else {
                                                tracing::debug!("View already exists: {}", view.label);
                                            }
                                        }
                                    }
                                    Err(element) => {
                                        tracing::debug!(
                                            "Failed to downcast element to views: {:?}",
                                            element
                                        );
                                    }
                                }
                            }
                        }

                        for (label, id) in lut {
                            if !new_labels.contains(&label) {
                                tracing::debug!("Removing view: {}", label);

                                if let Some(view) = self.state.views.remove(&id) {
                                    view.destroy();
                                }

                                self.state.lut.remove(&label);
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
        exclusive_zone: i32,
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
        layer.set_exclusive_zone(exclusive_zone);
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

    pub(crate) fn create_wayland_surface(&self, view: &View<Message>) -> SurfaceHandle {
        match view.window {
            true => {
                let window = self.create_window(
                    view.decorations,
                    view.label.clone(),
                    view.min_size,
                    view.max_size,
                );
                SurfaceHandle::Window(window)
            }
            false => {
                let layer = self.create_layer(
                    view.layer,
                    view.label.clone(),
                    view.anchor,
                    view.keyboard_interactivity,
                    view.size,
                    view.exclusive_zone,
                    view.margin,
                );
                SurfaceHandle::Layer(layer)
            }
        }
    }
}
