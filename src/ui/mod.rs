//! UI related modules

pub mod menu;
pub mod popup;
pub mod status;
pub mod touchpad;

pub mod prelude {
    pub use super::{
        menu::prelude::*, popup::prelude::*, status::prelude::*, touchpad::prelude::*,
    };
}
