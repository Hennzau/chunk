use std::sync::Arc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::prelude::*;

pub(crate) struct TaskPool<Message> {
    task_sender: UnboundedSender<Task<Message>>,
    task_receiver: UnboundedReceiver<Task<Message>>,
}

impl<Message: Sync + Send + 'static> TaskPool<Message> {
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded_channel();

        TaskPool {
            task_sender: tx,
            task_receiver: rx,
        }
    }

    pub(crate) fn sender(&self) -> UnboundedSender<Task<Message>> {
        self.task_sender.clone()
    }

    pub(crate) async fn run(
        mut self,
        on_error: impl Fn(Report) -> Message + 'static + Send + Sync,
        msg_client: UnboundedSender<Message>,
        directive_client: UnboundedSender<ApplicationDirective>,
    ) -> Result<()> {
        tracing::info!("TaskPool started");

        let on_error = Arc::new(on_error);

        while let Some(task) = self.task_receiver.recv().await {
            match task.handle {
                TaskHandle::Special(directive) => match directive {
                    SpecialTask::None => {}
                    directive => {
                        directive_client.send(match directive {
                            SpecialTask::Stop => ApplicationDirective::Stop,
                            SpecialTask::ResetState => ApplicationDirective::ResetState,
                            SpecialTask::None => unreachable!(),
                        })?;
                    }
                },
                TaskHandle::Simple(fut) => {
                    let result_sender = msg_client.clone();
                    let on_error = on_error.clone();

                    tokio::spawn(async move {
                        let result = fut.await;

                        result_sender
                            .send(result.map_or_else(|e| on_error(e), |msg| msg))
                            .unwrap_or_else(|e| {
                                tracing::error!("Failed to send message: {}", e);
                            });
                    });
                }
                TaskHandle::Batch(tasks) => {
                    for t in tasks {
                        self.task_sender.send(t)?;
                    }
                }
                TaskHandle::Then(first, second) => {
                    let tx = self.task_sender.clone();

                    let result_sender = msg_client.clone();
                    let on_error = on_error.clone();
                    tokio::spawn(async move {
                        let result = match first.handle {
                            TaskHandle::Simple(fut) => fut.await,
                            _ => {
                                tracing::error!("Tried to schedule a non-simple task.");
                                return;
                            }
                        };

                        result_sender
                            .send(result.map_or_else(|e| on_error(e), |msg| msg))
                            .unwrap_or_else(|e| {
                                tracing::error!("Failed to send message: {}", e);
                            });

                        tx.send(*second).unwrap_or_else(|e| {
                            tracing::error!("Failed to send next task: {}", e);
                        });
                    });
                }
            }
        }

        Ok(())
    }
}
