use std::time::Duration;

use kyo::prelude::{reexport::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    Application::new(State::default, State::update, State::render)
        .task(Task::msg(Message::Open))
        .run::<WaylandBackend<Message>>(|e| {
            tracing::error!("Error in application: {:?}", e);

            Message::Stop
        })
        .await
}

enum Message {
    Stop,
    Open,
    Nothing,
}

struct State {
    top: bool,
}

impl Default for State {
    fn default() -> Self {
        Self { top: true }
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Nothing => {
                self.top = false;

                Task::none()
            }
            Message::Stop => Task::stop(),
            Message::Open => Task::submit(empty().label("bar.left").layout(Layout {
                width: 24,
                height: 1080,
                reserve: Some(Reserve::Left),

                ..Default::default()
            }))
            .then(Task::wait(Duration::from_millis(1000), Message::Nothing))
            .then(Task::close("bar.left")),
        }
    }

    fn render(&self) -> Element<Message> {
        let elements = container().with(empty().label("bar.bottom").layout(Layout {
            width: 1920,
            height: 24,
            reserve: Some(Reserve::Top),

            ..Default::default()
        }));

        match self.top {
            true => elements.with(empty().label("bar.top").layout(Layout {
                width: 1920,
                height: 24,
                reserve: Some(Reserve::Bottom),

                ..Default::default()
            })),
            false => elements,
        }
        .element()
    }
}
