pub mod event;

pub mod pool;
pub mod task;

pub mod element;
pub mod widget;

pub mod application;
pub mod backend;

pub mod prelude {
    pub use eyre::{Report, Result};

    pub use crate::event::*;

    pub(crate) use crate::pool::*;
    pub use crate::task::*;

    pub use crate::element::*;
    pub use crate::widget::*;

    pub use crate::application::*;
    pub use crate::backend::*;

    pub(crate) use reexport::*;

    pub mod reexport {
        pub use chii::prelude::*;
    }
}
