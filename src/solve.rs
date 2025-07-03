use std::collections::HashMap;

use crate::{
    game::{possible_moves, GameState, Move, Roll},
    solve::table::Table,
};

mod table;

pub fn solve() -> (HashMap<GameState, u32>, Vec<GameState>, Table) {
    println!("creating state mappings...");
    let (state_indices, states) = get_mappings();
    println!("number of states: {}", states.len());

    println!("creating table...");
    let mut table = Table::new(&state_indices, &states);

    let epsilon = 1e-6;
    let mut delta = f64::INFINITY;
    while delta.abs() >= epsilon {
        delta = table.converge();
    }

    (state_indices, states, table)
}

fn get_mappings() -> (HashMap<GameState, u32>, Vec<GameState>) {
    let mut state_indices = HashMap::new();
    let mut states = Vec::new();
    let mut state_queue = vec![GameState::new()];

    while let Some(game) = state_queue.pop() {
        get_mappings_rec(game, &mut state_queue, &mut state_indices, &mut states);
    }
    (state_indices, states)
}
fn get_mappings_rec(
    game: GameState,
    state_queue: &mut Vec<GameState>,
    state_indices: &mut HashMap<GameState, u32>,
    states: &mut Vec<GameState>,
) {
    if states.len() % 100000 == 0 {
        println!("created {} mappings", states.len());
    }
    if state_indices.contains_key(&game) {
        return;
    } else {
        let index = states.len();
        states.push(game);
        state_indices.insert(game, index.try_into().expect("too many game states"));
    }
    for mov in Roll::iter_all().flat_map(|roll| possible_moves(game, roll)) {
        if let Move::Continue { game, keep_turn } = mov {
            state_queue.push(if keep_turn { game } else { game.flipped() });
        }
    }
}
