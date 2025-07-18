use std::pin::Pin;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::prelude::*;

pub trait BackendTrait<Message>: Send + Sync {
    fn new() -> impl Future<Output = Result<Self>> + Send + 'static
    where
        Self: Sized;

    fn client(&self) -> UnboundedSender<Element<Message>>;

    fn run(
        self,
        client: UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>
    where
        Self: Send;
}

pub struct EmptyBackend<Message> {
    pub(crate) client: UnboundedSender<Element<Message>>,
    pub(crate) server: UnboundedReceiver<Element<Message>>,
}

impl<Message: 'static> BackendTrait<Message> for EmptyBackend<Message> {
    fn new() -> impl Future<Output = Result<Self>> + Send + 'static {
        async move {
            let (client, server) = tokio::sync::mpsc::unbounded_channel::<Element<Message>>();
            Ok(Self { client, server })
        }
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
