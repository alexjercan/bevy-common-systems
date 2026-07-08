//! UI related modules

pub mod animate;
pub mod health_display;
pub mod menu;
pub mod objectives;
pub mod popup;
pub mod status;
pub mod touchpad;

pub mod prelude {
    pub use super::{
        animate::prelude::*, health_display::prelude::*, menu::prelude::*, objectives::prelude::*,
        popup::prelude::*, status::prelude::*, touchpad::prelude::*,
    };
}
