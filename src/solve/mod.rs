use core::f32;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use rustc_hash::FxHashMap as HashMap;

use crate::{
    game::{GameState, GameStateSmall, GOAL_SCORE},
    save,
    solve::{converge::converge, converge_gpu::DeviceHolder, order::get_order, perma::PermaKey},
};

mod converge;
mod converge_gpu;
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

pub fn solve() -> (Vec<GameStateSmall>, Vec<f32>) {
    let (states, perma_keys) = get_order();
    println!("number of states: {}", states.len());
    println!("number of perma keys: {}", perma_keys.len());

    let mut vals = vec![-1.0; states.len()];

    let mut device_holder = DeviceHolder::new();

    let mut expr_parts = Vec::new();
    let mut expr_starts = Vec::new();

    for (i, (key, range)) in perma_keys.iter().enumerate() {
        let lowest_dep = (&perma_keys[..=i])
            .iter()
            .rev()
            .filter(|(other_key, _)| other_key.reachable_in_one_move_from(*key))
            .last()
            .unwrap();
        let dep_start = lowest_dep.1.start;
        println!(
            "dep score {} {}, score {} {}",
            lowest_dep.0.team_gt.score,
            lowest_dep.0.team_lt.score,
            key.team_gt.score,
            key.team_lt.score
        );
        converge(
            &states,
            &mut vals,
            dep_start,
            range.start,
            range.end,
            &mut device_holder,
            &mut expr_parts,
            &mut expr_starts,
        );
    }
    save_vals(&vals, 0);

    (states, vals)
}

pub fn save_vals(vals: &[f32], converge_count: usize) {
    println!("saving vals...");
    save::write(
        &format!("./data/vals_{}_{}.bin", GOAL_SCORE, converge_count),
        vals,
    );
}
