use std::sync::Arc;

use crate::prelude::*;

pub struct Application<State, Message> {
    pub(crate) state: Box<State>,

    pub(crate) update: Box<dyn Fn(&mut State, Message) -> Option<Task<Message>>>,
    pub(crate) render: Box<dyn Fn(&State) -> Element<Message>>,
}

impl<State, Message> Application<State, Message> {
    pub async fn new(
        state: State,
        update: impl Fn(&mut State, Message) -> Option<Task<Message>> + 'static,
        render: impl Fn(&State) -> Element<Message> + 'static,
    ) -> Result<Self> {
        Ok(Self {
            state: Box::new(state),
            update: Box::new(update),
            render: Box::new(render),
        })
    }

    pub async fn run(self, on_error: impl Fn(Report) -> Message + 'static) -> Result<()> {
        let _on_error = Arc::new(on_error);

        Ok(())
    }
}
