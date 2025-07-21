pub mod backend;

pub mod widget;

pub mod prelude {
    pub use eyre::{Report, Result};

    pub use crate::backend::*;
    pub use crate::widget::*;

    pub(crate) use reexport::*;
    pub mod reexport {
        pub use hej::prelude::{reexport::*, *};
    }
}
