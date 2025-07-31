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
}

#[derive(Default)]
struct State {}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Stop => Task::stop(),
            Message::Open => Task::submit(empty().label("bar.left").layout(Layout {
                width: 24,
                height: 1080,
                reserve: Some(Reserve::Left),

                ..Default::default()
            })),
        }
    }

    fn render(&self) -> Element<Message> {
        container()
            .with(empty().label("bar.bottom").layout(Layout {
                width: 1920,
                height: 24,
                reserve: Some(Reserve::Top),

                ..Default::default()
            }))
            .with(empty().label("bar.top").layout(Layout {
                width: 1920,
                height: 24,
                reserve: Some(Reserve::Bottom),

                ..Default::default()
            }))
            .element()
    }
}
