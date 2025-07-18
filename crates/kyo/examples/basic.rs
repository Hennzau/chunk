// use kyo::prelude::{reexport::*, *};

// use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {}

// #[tokio::main]
// async fn main() -> Result<()> {
//     Application::new(State::default(), State::update, State::render)
//         .await?
//         .run(|report| Message::Error(Arc::new(report)))
//         .await
// }

// #[derive(Debug, Clone)]
// pub enum Message {
//     EscapePressed,

//     Stop,

//     Error(Arc<Report>),
// }

// pub struct State {}

// impl Default for State {
//     fn default() -> Self {
//         Self {}
//     }
// }

// impl State {
//     fn update(&mut self, message: Message) -> Option<Task<Message>> {
//         match message {
//             Message::EscapePressed => {
//                 println!("Escape key pressed");

//                 Task::some(async move {
//                     tokio::time::sleep(Duration::from_millis(500)).await;

//                     Ok(Message::Stop)
//                 })
//             }
//             Message::Stop => {
//                 println!("Stopping the application...");

//                 Task::stop()
//             }
//             Message::Error(report) => {
//                 eprintln!("Error occurred: {}", report);

//                 Task::stop()
//             }
//         }
//     }

//     fn render(&self) -> Element<Message> {
//         Element::empty()
//     }
// }
