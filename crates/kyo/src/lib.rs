pub mod application;

pub mod prelude {
    pub use eyre::{Report, Result};

    pub use crate::application::*;

    pub(crate) use reexport::*;
    pub mod reexport {
        pub use chii::prelude::*;
        pub use hej::prelude::*;
    }
}
