use std::pin::Pin;

use crate::prelude::*;

pub trait Widget<Message>: Sync {
    fn on_event<'a>(
        &'a mut self,
        event: &'a Event,
        client: &'a Client<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    where
        Self: Sync + 'a;

    fn render(&self, renderer: &mut Renderer) -> Result<()>;
}
