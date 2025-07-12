use std::ops::Range;

use crate::game::GameStateSmall;

pub fn converge(states: &[GameStateSmall], vals: &mut [f32], range: Range<usize>) {
    let states = &states[range];
}
