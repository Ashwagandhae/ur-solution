use std::time::{Duration, Instant};

use rustc_hash::FxHashMap as HashMap;

use crate::{
    game::{GameState, GOAL_SCORE},
    save,
    solve::{mapping::get_mappings, table::Table, table_gpu::TableGpu},
};

pub mod expr;
pub mod mapping;
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
    let (state_indices, states) = get_mappings();
    println!("number of states: {}", states.len());

    println!("creating table...");
    let mut table = Table::new(&state_indices, &states);
    println!("created table");

    let mut last_save = Instant::now();

    const NUM_CONVERGE: usize = 1;

    const SAVE_INTERVAL: Duration = Duration::from_secs(3 * 60);

    let mut converge_count = 0;
    loop {
        time_it(&format!("converge {NUM_CONVERGE} times"), || {
            table.converge();
        });
        converge_count += NUM_CONVERGE;
        println!("converged {converge_count} times");
        table.stats();

        if last_save.elapsed() > SAVE_INTERVAL {
            last_save = Instant::now();
            save_vals(&table, converge_count);
        }
    }
}

pub fn save_vals(table: &Table, converge_count: usize) {
    println!("saving vals...");
    save::write(
        &format!("./data/vals_{}_{}.bin", GOAL_SCORE, converge_count),
        table.vals(),
    );
}
