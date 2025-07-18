pub mod event;

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
        //! A collection of re-exports for commonly used types and traits.

        pub use chii::prelude::*;
    }
}
