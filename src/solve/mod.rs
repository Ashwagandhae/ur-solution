use std::time::Instant;

use crate::{
    game::{GameStateSmall, GOAL_SCORE},
    save,
    solve::{converge::converge, converge_gpu::DeviceHolder, order::get_order},
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
    println!("{label} took {elapsed:.3?}");
    result
}

pub fn solve() -> (Vec<GameStateSmall>, Vec<f64>) {
    let (states, perma_keys) = get_order();
    println!("number of states: {}", states.len());
    println!("number of perma keys: {}", perma_keys.len());

    let mut vals = vec![-1.0; states.len()];

    let mut device_holder = DeviceHolder::new();

    let mut expr_parts = Vec::new();
    let mut expr_starts = Vec::new();

    time_it("converge loop", || {
        for (i, (key, range)) in perma_keys.iter().enumerate() {
            let lowest_dep = perma_keys[..=i]
                .iter()
                .rev()
                .filter(|(other_key, _)| other_key.reachable_in_one_move_from(*key))
                .next_back()
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
    });

    save_vals(&vals, 0);

    (states, vals)
}

pub fn save_vals(vals: &[f64], converge_count: usize) {
    println!("saving vals...");
    save::write(
        &format!("./data/vals_{GOAL_SCORE}_{converge_count}.bin"),
        vals,
    );
}
