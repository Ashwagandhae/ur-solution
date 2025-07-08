use rustc_hash::FxHashMap as HashMap;

use crate::{
    game::{GameState, Move, PossibleMovesIter, Roll, GOAL_SCORE},
    save::read_or_create,
    successor::Succ,
};

pub fn get_mappings() -> (HashMap<GameState, u32>, Vec<GameState>) {
    read_or_create(
        &format!("./data/mappings_{}.bin", GOAL_SCORE),
        create_mappings,
        |(_, states)| states.iter().map(|state| (*state).into()).collect(),
        |data: &Vec<u64>| {
            let state_indices = data
                .iter()
                .enumerate()
                .map(|(i, val)| (GameState::from(*val), i.try_into().expect("too many state")))
                .collect();
            let states = data.iter().cloned().map(GameState::from).collect();
            (state_indices, states)
        },
    )
}
pub fn create_mappings() -> (HashMap<GameState, u32>, Vec<GameState>) {
    println!("creating mappings...");
    let mut state_indices = HashMap::default();
    let mut states = Vec::new();
    let mut state_queue = vec![GameState::new()];

    while let Some(game) = state_queue.pop() {
        create_mappings_rec(game, &mut state_queue, &mut state_indices, &mut states);
    }
    (state_indices, states)
}
fn create_mappings_rec(
    game: GameState,
    state_queue: &mut Vec<GameState>,
    state_indices: &mut HashMap<GameState, u32>,
    states: &mut Vec<GameState>,
) {
    if state_indices.contains_key(&game) {
        return;
    }
    let index = states.len();
    states.push(game);
    state_indices.insert(game, index.try_into().expect("too many game states"));
    if states.len() % 1_000_000 == 0 {
        println!("created {} mappings", states.len());
    }
    state_queue.extend(
        Roll::succ_iter()
            .flat_map(|roll| PossibleMovesIter::new(game, roll))
            .filter_map(|mov| {
                if let Move::Continue { game, keep_turn } = mov {
                    Some(if keep_turn { game } else { game.flipped() })
                } else {
                    None
                }
            })
            .filter(|game| !state_indices.contains_key(&game)),
    );
}
