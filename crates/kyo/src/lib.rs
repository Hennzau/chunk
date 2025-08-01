pub mod backend;
pub(crate) mod surface;
pub(crate) mod widget;

pub mod prelude {
    pub use eyre::{Report, Result};

    pub use crate::backend::*;
    pub(crate) use crate::surface::*;
    pub(crate) use crate::widget::*;

    pub(crate) use reexport::*;
    pub mod reexport {
        pub use hej::prelude::{reexport::*, *};
    }
}
