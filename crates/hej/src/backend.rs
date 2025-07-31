//! This module defines a backend trait and an empty backend implementation.

use std::pin::Pin;

use crate::prelude::*;

/// The BackendTrait defines the interface for a backend that can receives elements to render and
/// manage.
pub trait Backend<Message>: Send + Sync {
    /// Creates a new instance of the backend.
    fn new() -> impl Future<Output = Result<Self>> + Send + 'static
    where
        Self: Sized;

    /// Returns a clone of the client sender that can be used to send elements to the backend externally.
    fn submitter(&self) -> Submitter<Element<Message>>;

    fn closer(&self) -> Submitter<String>;

    /// Runs the backend, processing elements and handling messages.
    fn run(self, client: Submitter<Message>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
    where
        Self: Send;
}

/// An empty backend implementation that does not perform any operations.
pub struct EmptyBackend<Message> {
    pub(crate) submitter: Submitter<Element<Message>>,
    pub(crate) server: Server<Element<Message>>,

    pub(crate) closer: Submitter<String>,
    pub(crate) _closer_server: Server<String>,
}

impl<Message: 'static> Backend<Message> for EmptyBackend<Message> {
    async fn new() -> Result<Self> {
        let (submitter, server) = channel();
        let (closer, _closer_server) = channel();

        Ok(Self {
            submitter,
            server,
            closer,
            _closer_server,
        })
    }

    fn closer(&self) -> Submitter<String> {
        self.closer.clone()
    }

    fn submitter(&self) -> Submitter<Element<Message>> {
        self.submitter.clone()
    }

    fn run(
        mut self,
        _client: Submitter<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        Box::pin(async move {
            tracing::info!("Backend started");

            while let Ok(_element) = self.server.recv().await {
                tracing::debug!("Received element");
            }
            Ok(())
        })
    }
}
