use std::{pin::Pin, sync::Arc};

use hej::prelude::{reexport::*, *};
use tokio::runtime::Handle;

#[derive(Debug)]
pub enum Message {
    Nothing,
    Error(Report),
}

pub struct Test {}

impl Widget<Message> for Test {
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

#[tokio::main]
async fn main() -> Result<()> {
    let (client, _server) = channel::<Message>();

    let task = Task::new(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("Task completed after 1 second");

        Ok(Message::Nothing)
    })
    .then(
        Task::new(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            println!("Task completed after 2 seconds");

            Ok(Message::Nothing)
        })
        .batch(Task::new(async move {
            tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
            println!("Task completed after 2.5 seconds");

            Ok(Message::Nothing)
        })),
    );

    let handle = Handle::current();
    match task.execute(handle, Arc::new(client), Arc::new(Message::Error)) {
        Status::TaskRunning(handle) => handle,
        Status::TaskAskedForStop => return Ok(()),
    }
    .await?;

    Ok(())
}
