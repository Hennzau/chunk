use std::pin::Pin;

use crate::prelude::*;

pub struct Element<Message> {
    pub(crate) widget: Box<dyn Widget<Message>>,
}

impl<Message> Element<Message> {
    pub fn empty() -> Self {
        EmptyWidget {}.element()
    }

    pub async fn on_event(&mut self, event: &Event, client: &Client<Message>) -> Result<()> {
        self.widget.on_event(event, client).await
    }

    pub fn render(&self, renderer: &mut Renderer) -> Result<()> {
        self.widget.render(renderer)
    }
}

pub trait IntoElement<Message> {
    fn element(self) -> Element<Message>;
}

impl<Message, T> IntoElement<Message> for T
where
    T: Widget<Message> + 'static,
{
    fn element(self) -> Element<Message> {
        Element {
            widget: Box::new(self),
        }
    }
}

pub struct EmptyWidget {}

impl<Message> Widget<Message> for EmptyWidget {
    fn on_event<'a>(
        &'a mut self,
        _event: &'a Event,
        _client: &'a Client<Message>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
    where
        Self: Sync + 'a,
    {
        Box::pin(async move { Ok(()) })
    }

    fn render(&self, _renderer: &mut Renderer) -> Result<()> {
        Ok(())
    }
}

pub fn empty() -> EmptyWidget {
    EmptyWidget {}
}
