pub const MAX_MAIN: u32 = 1;
pub const RATIO: f32 = 0.6;
pub const RATIO_STEP: f32 = 0.1;
pub const OUTER_PX: u32 = 10;
pub const INNER_PX: u32 = 10;
pub const BAR_HEIGHT_PX: u32 = 30;
pub const PANEL_HEIGHT_PX: u32 = 30;

use penrose::{core::bindings::KeyEventHandler, x11rb::RustConn};
pub type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

pub mod actions;
pub mod bindings;
pub mod layouts;
