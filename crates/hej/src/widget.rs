use std::pin::Pin;

use tokio::sync::mpsc::UnboundedSender;

use crate::prelude::*;

pub trait Widget<Message>: Send + Sync {
    fn on_event<'a>(
        &'a mut self,
        event: &'a Event,
        client: &'a UnboundedSender<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    where
        Self: Sync + 'a;

    fn render(&self, renderer: &mut Renderer) -> Result<()>;
}
