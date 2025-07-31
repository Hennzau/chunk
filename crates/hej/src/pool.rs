//! This module defines a `TaskPool` that manages and executes tasks asynchronously.
//! It allows for sending tasks that can be simple, batched, or chained together, and
//! handles special tasks like stopping the application or resetting the state.

use std::sync::Arc;

use crate::prelude::*;

pub(crate) struct TaskPool<Message> {
    submitter: Submitter<Task<Message>>,
    server: Server<Task<Message>>,
}

impl<Message: Sync + Send + 'static> TaskPool<Message> {
    pub(crate) fn new() -> Self {
        let (submitter, server) = channel();

        TaskPool { submitter, server }
    }

    pub(crate) fn submitter(&self) -> Submitter<Task<Message>> {
        self.submitter.clone()
    }

    pub(crate) async fn run(
        mut self,
        on_error: impl Fn(Report) -> Message + 'static + Send + Sync,
        msg_submitter: Submitter<Message>,
        directive_submitter: Submitter<ApplicationDirective<Message>>,
    ) {
        tracing::info!("TaskPool started");

        let on_error = Arc::new(on_error);

        while let Ok(task) = self.server.recv().await {
            let signal = task.signal;

            match task.handle {
                TaskHandle::Special(directive) => {
                    match directive {
                        SpecialTask::None => {}
                        directive => {
                            directive_submitter
                                .submit(match directive {
                                    SpecialTask::Stop => ApplicationDirective::Stop,
                                    SpecialTask::ResetState => ApplicationDirective::ResetState,
                                    SpecialTask::Submit(element) => {
                                        ApplicationDirective::Submit(element)
                                    }
                                    SpecialTask::Close(label) => ApplicationDirective::Close(label),
                                    SpecialTask::None => unreachable!(),
                                })
                                .unwrap_or_else(|e| {
                                    tracing::error!("Failed to send directive: {}", e);
                                });
                        }
                    }

                    signal.map(|s| s.send(()));
                }
                TaskHandle::Simple(fut) => {
                    let result_sender = msg_submitter.clone();
                    let on_error = on_error.clone();
                    tokio::spawn(async move {
                        let result = fut.await;
                        signal.map(|s| s.send(()));

                        result_sender
                            .submit(result.unwrap_or_else(|e| on_error(e)))
                            .unwrap_or_else(|e| {
                                tracing::error!("Failed to send message: {}", e);
                            });
                    });
                }
                TaskHandle::Batch(tasks) => {
                    let tx = self.submitter.clone();
                    tokio::spawn(async move {
                        let mut releases = Vec::new();

                        for mut t in tasks {
                            let (tsignal, release) = tokio::sync::oneshot::channel();

                            t.signal = Some(tsignal);

                            tx.submit(t).unwrap_or_else(|e| {
                                tracing::error!("Failed to send task: {}", e);
                            });

                            releases.push(release);
                        }

                        for release in releases {
                            release.await.unwrap_or_else(|e| {
                                tracing::error!("Failed to release task: {}", e);
                            });
                        }

                        signal.map(|s| s.send(()));
                    });
                }
                TaskHandle::Then(mut first, mut second) => {
                    let tx = self.submitter.clone();
                    tokio::spawn(async move {
                        let (fsignal, release) = tokio::sync::oneshot::channel();
                        first.signal = Some(fsignal);

                        tx.submit(*first).unwrap_or_else(|e| {
                            tracing::error!("Failed to send first task: {}", e);
                        });

                        release.await.unwrap_or_else(|e| {
                            tracing::error!("Failed to release first task: {}", e);
                        });

                        let (ssignal, release) = tokio::sync::oneshot::channel();
                        second.signal = Some(ssignal);
                        tx.submit(*second).unwrap_or_else(|e| {
                            tracing::error!("Failed to send second task: {}", e);
                        });

                        release.await.unwrap_or_else(|e| {
                            tracing::error!("Failed to release second task: {}", e);
                        });

                        signal.map(|s| s.send(()));
                    });
                }
            }
        }
    }
}
