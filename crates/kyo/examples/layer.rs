use kyo::prelude::{reexport::*, *};

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
        empty()
            .label("menu")
            .layout(Layout {
                width: 720,
                height: 480,
                x: 1920 / 2 - 720 / 2,
                y: 1080 / 2 - 480 / 2,
                keyboard_sensitivity: KeyboardSensitivity::OnClick,
                ..Default::default()
            })
            .element()
    }
}
