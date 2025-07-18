//! This module defines a backend trait and an empty backend implementation.

use std::pin::Pin;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::prelude::*;

/// The BackendTrait defines the interface for a backend that can receives elements to render and
/// manage.
pub trait BackendTrait<Message>: Send + Sync {
    /// Creates a new instance of the backend.
    fn new() -> impl Future<Output = Result<Self>> + Send + 'static
    where
        Self: Sized;

    /// Returns a clone of the client sender that can be used to send elements to the backend externally.
    fn client(&self) -> UnboundedSender<Element<Message>>;

    /// Runs the backend, processing elements and handling messages.
    fn run(
        self,
        client: UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
    where
        Self: Send;
}

/// An empty backend implementation that does not perform any operations.
pub struct EmptyBackend<Message> {
    pub(crate) client: UnboundedSender<Element<Message>>,
    pub(crate) server: UnboundedReceiver<Element<Message>>,
}

impl<Message: 'static> BackendTrait<Message> for EmptyBackend<Message> {
    async fn new() -> Result<Self> {
        let (client, server) = tokio::sync::mpsc::unbounded_channel::<Element<Message>>();
        Ok(Self { client, server })
    }

    fn client(&self) -> UnboundedSender<Element<Message>> {
        self.client.clone()
    }

    fn run(
        mut self,
        _client: UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            tracing::info!("Backend started");

            while let Some(_element) = self.server.recv().await {
                tracing::debug!("Received element");
            }
            Ok(())
        })
    }
}
