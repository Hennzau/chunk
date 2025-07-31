use std::sync::Arc;

pub struct Map<MessageA, MessageB> {
    pub(crate) convert: Arc<dyn Fn(MessageA) -> MessageB + 'static + Send + Sync>,
}

impl<MessageA, MessageB> Map<MessageA, MessageB> {
    pub fn new(convert: impl Fn(MessageA) -> MessageB + 'static + Send + Sync) -> Self {
        Self {
            convert: Arc::new(convert),
        }
    }

    pub fn map(&self, message: MessageA) -> MessageB {
        (self.convert)(message)
    }

    pub fn clone(&self) -> Self {
        Self {
            convert: Arc::clone(&self.convert),
        }
    }
}
