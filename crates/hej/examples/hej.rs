use std::{sync::Arc, time::Duration};

use hej::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    Application::new(State::default, State::update, State::render)
        .await?
        .run::<EmptyBackend<Message>>(|e| Message::Error(Arc::new(e)))
        .await
}

enum Message {
    Nothing,
    Error(Arc<Report>),
}

struct State {}

impl Default for State {
    fn default() -> Self {
        State {}
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Nothing => Task::wait(Duration::from_millis(100), Message::Nothing),
            Message::Error(report) => {
                tracing::error!("An error occurred: {}", report);
                Task::none()
            }
        }
    }

    fn render(&self) -> Element<Message> {
        Element::empty()
    }
}
