pub mod event;
pub mod message;

pub mod task;

pub mod element;
pub mod widget;

pub mod prelude {
    pub use eyre::{Report, Result};

    pub use crate::event::*;
    pub use crate::message::*;

    pub use crate::task::*;

    pub use crate::element::*;
    pub use crate::widget::*;

    pub(crate) use reexport::*;

    pub mod reexport {
        pub use chii::prelude::*;
    }
}
