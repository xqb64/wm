pub const MAX_MAIN: u32 = 1;
pub const RATIO: f32 = 0.6;
pub const RATIO_STEP: f32 = 0.1;

use penrose::{core::bindings::KeyEventHandler, x11rb::RustConn};
pub type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

pub mod actions;
pub mod bindings;
pub mod layouts;
