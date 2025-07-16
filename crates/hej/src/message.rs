use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::prelude::*;

pub struct Client<Message> {
    pub(crate) sender: UnboundedSender<Message>,
}

impl<Message: 'static + Send + Sync> Client<Message> {
    pub fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }

    pub fn send(&self, message: Message) -> Result<()> {
        self.sender.send(message).map_err(Report::msg)
    }

    pub fn send_no_result(&self, message: Message) {
        if let Err(e) = self.send(message) {
            eprintln!("Failed to send message: {}", e);
        }
    }
}

pub struct Server<Message> {
    pub(crate) receiver: UnboundedReceiver<Message>,
}

impl<Message> Server<Message> {
    pub async fn recv(&mut self) -> Result<Message> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| Report::msg("Receiver closed"))
    }
}

pub fn channel<Message>() -> (Client<Message>, Server<Message>) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    (Client { sender }, Server { receiver })
}
