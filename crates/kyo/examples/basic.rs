use kyo::prelude::{reexport::*, *};
use smithay_client_toolkit::shell::wlr_layer::Anchor;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    Application::new(State::default, State::update, State::render)
        .run::<WaylandBackend<Message>>(|e| {
            tracing::error!("Error in application: {:?}", e);

            Message::Stop
        })
        .await
}

enum Message {
    Stop,
}

#[derive(Default)]
struct State {}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Stop => Task::stop(),
        }
    }

    fn render(&self) -> Element<Message> {
        views()
            .view(view().anchor(Anchor::BOTTOM).label("bar.bottom"))
            .view(view().anchor(Anchor::TOP).label("bar.top"))
            .element()
    }
}
