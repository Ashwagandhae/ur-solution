use core::f32;
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap as HashMap;

use crate::{
    game::{GameState, GOAL_SCORE},
    save,
    solve::{order::get_order, table::Table, table_gpu::TableGpu},
};

mod converge;
pub mod expr;
pub mod order;
pub mod perma;
mod table;
pub mod table_gpu;

pub fn time_it<F, R>(label: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    println!("{label} took {:.3?}", elapsed);
    result
}

pub fn solve() {
    let states = get_order();
    println!("number of states: {}", states.len());
    let mut vals = vec![f32::NAN; states.len()];
}

pub fn save_vals(table: &Table, converge_count: usize) {
    println!("saving vals...");
    save::write(
        &format!("./data/vals_{}_{}.bin", GOAL_SCORE, converge_count),
        table.vals(),
    );
}
