use std::{
    collections::{BTreeMap, HashSet},
    ops::Range,
};

use itertools::Itertools;

use crate::{
    game::{GameState, GameStateSmall, Move, PossibleMovesIter, Roll, GOAL_SCORE},
    save::{self, read_or_create},
    solve::perma::PermaKey,
    successor::Succ,
};
use rayon::prelude::*;

pub fn get_order() -> (Vec<GameStateSmall>, Vec<(PermaKey, Range<usize>)>) {
    let mut states = read_or_create(
        &format!("./data/order_{}.bin", GOAL_SCORE),
        create_order,
        |states| states.iter().map(|state| (*state).into()).collect(),
        |data: &Vec<u32>| data.iter().cloned().map(GameStateSmall::from).collect(),
    );

    let mut counts: BTreeMap<PermaKey, usize> = BTreeMap::new();
    for game in &states {
        let key = PermaKey::from(game);
        *counts.entry(key).or_insert(0) += 1;
    }

    let perma_keys = counts.into_iter().sorted().fold(
        Vec::new(),
        |mut acc: Vec<(PermaKey, Range<usize>)>, (key, count)| {
            let start = acc.last().map(|(_, range)| range.end).unwrap_or(0);
            acc.push((key, start..(start + count)));
            acc
        },
    );

    let is_sorted = states.is_sorted_by(|x, y| x < y);
    println!("is sorted: {}", is_sorted);
    if !is_sorted {
        println!("sorting...");
        states.par_sort();
        save::write(
            &format!("./data/order_{}.bin", GOAL_SCORE),
            states.iter().map(|state| u32::from(*state)).collect_vec(),
        );
    }

    (states, perma_keys)
}
pub fn create_order() -> Vec<GameStateSmall> {
    println!("creating order...");
    let mut states = HashSet::new();
    let mut state_queue = vec![GameState::new().into()];

    while let Some(game) = state_queue.pop() {
        create_order_rec(game, &mut state_queue, &mut states);
    }

    let mut states: Vec<_> = states.into_iter().collect();
    states.par_sort();

    states
}
fn create_order_rec(
    game: GameStateSmall,
    state_queue: &mut Vec<GameStateSmall>,
    states: &mut HashSet<GameStateSmall>,
) {
    if states.contains(&game) {
        return;
    }
    states.insert(game);

    if states.len() % 1_000_000 == 0 {
        println!("created {} order", states.len());
    }
    state_queue.extend(
        game_deps(GameState::from(game))
            .map(GameStateSmall::from)
            .filter(|new_game| !states.contains(&new_game)),
    );
}

fn game_deps(game: GameState) -> impl Iterator<Item = GameState> {
    Roll::succ_iter()
        .flat_map(move |roll| PossibleMovesIter::new(game, roll))
        .filter_map(|mov| {
            if let Move::Continue { game, keep_turn } = mov {
                Some(if keep_turn { game } else { game.flipped() })
            } else {
                None
            }
        })
}
