pub mod event;

pub mod map;

pub(crate) mod pool;
pub mod task;

pub mod element;
pub mod widget;

pub mod application;
pub mod backend;

pub mod prelude {
    //! A collection of commonly used types and traits for making usable applications.
    //! A developer should only need to import this module to access all functionality.
    //!
    //! Note: You may need to also import the `reexport` module for additional types when
    //! dealing with widgets.

    pub use crate::event::*;
    pub use eyre::{Report, Result};

    pub use crate::map::*;

    pub(crate) use crate::pool::*;
    pub use crate::task::*;

    pub use crate::element::*;
    pub use crate::widget::*;

    pub use crate::application::*;
    pub use crate::backend::*;

    pub(crate) use reexport::*;

    pub mod reexport {
        //! A collection of re-exports for commonly used types and traits.

        pub use chii::prelude::*;
    }

    use eyre::OptionExt;
    use tokio::sync::mpsc::UnboundedReceiver;
    use tokio::sync::mpsc::UnboundedSender;
    use tokio::sync::mpsc::unbounded_channel;

    pub struct Submitter<T> {
        pub(crate) tx: UnboundedSender<T>,
    }

    impl<T: 'static + Send + Sync> Submitter<T> {
        pub fn new(tx: UnboundedSender<T>) -> Self {
            Self { tx }
        }

        pub fn submit(&self, v: T) -> Result<()> {
            self.tx.send(v).map_err(Report::msg)
        }

        pub fn clone(&self) -> Self {
            Self {
                tx: self.tx.clone(),
            }
        }
    }

    pub struct Server<T> {
        pub(crate) rx: UnboundedReceiver<T>,
    }

    impl<T: 'static + Send + Sync> Server<T> {
        pub fn new(rx: UnboundedReceiver<T>) -> Self {
            Self { rx }
        }

        pub async fn recv(&mut self) -> Result<T> {
            self.rx.recv().await.ok_or_eyre("Channel Closed")
        }

        pub fn try_recv(&mut self) -> Result<T> {
            self.rx.try_recv().map_err(Report::msg)
        }
    }

    pub fn channel<T: 'static + Send + Sync>() -> (Submitter<T>, Server<T>) {
        let (tx, rx) = unbounded_channel();

        (Submitter::new(tx), Server::new(rx))
    }
}
