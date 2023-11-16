use crate::{MAX_MAIN, RATIO, RATIO_STEP};
use penrose::{
    builtin::layout::{Grid, MainAndStack, Monocle},
    core::layout::LayoutStack,
    extensions::layout::Tatami,
    stack,
};

pub fn layouts() -> LayoutStack {
    stack!(
        MainAndStack::side(MAX_MAIN, RATIO, RATIO_STEP),
        Tatami::boxed(RATIO, RATIO_STEP),
        Grid::boxed(),
        Monocle::boxed()
    )
}
