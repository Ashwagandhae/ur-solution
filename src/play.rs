use itertools::Itertools;
use std::collections::HashMap;

use crate::{
    game::{
        possible_moves,
        strip::{Delta, MoveSource},
        GameState, Move, Roll,
    },
    render::render,
};

#[derive(Debug, Clone, Copy)]
pub enum Chance {
    Calculating,
    Of(f32),
}

pub fn play() -> HashMap<GameState, Chance> {
    let mut memo = HashMap::new();
    play_rec(GameState::new(), &mut memo);
    memo
}

pub fn play_rec(game: GameState, memo: &mut HashMap<GameState, Chance>) -> Option<f32> {
    match memo.get(&game) {
        Some(Chance::Of(c)) => Some(*c),
        Some(Chance::Calculating) => None, // prevent loops
        None => {
            memo.insert(game, Chance::Calculating);
            // println!("{}", render(&game));
            // println!("calculating");
            // println!("have found {} games", memo.len());
            let chances: Vec<_> = Roll::iter_all()
                .filter_map(|roll| {
                    let possible_moves = possible_moves(game, roll);

                    let best_chance = possible_moves
                        .into_iter()
                        .filter_map(|mov| match mov {
                            Move::End { .. } => Some(1.0),
                            Move::Continue {
                                game, keep_turn, ..
                            } => {
                                if keep_turn {
                                    play_rec(game, memo)
                                } else {
                                    play_rec(game.flipped(), memo).map(|x| 1.0 - x)
                                }
                            }
                        })
                        .reduce(f32::max);
                    best_chance.map(|chance| (chance, roll.weight()))
                })
                .collect();
            // if chances.is_empty() {
            //     println!("{}", render(&game));
            //     panic!("chances is empty");
            // }
            let total_weight: f32 = chances.iter().map(|(_, weight)| weight).sum();
            let chance = if chances.is_empty() {
                panic!("total weight is zero");
            } else {
                chances
                    .iter()
                    .map(|(chance, weight)| chance * weight)
                    .sum::<f32>()
                    / total_weight
            };

            memo.insert(game, Chance::Of(chance));
            // println!("{}", render(&game));
            // println!("found win chance of game to be {}", chance);
            println!("have found {} games", memo.len());
            Some(chance)
        }
    }
}
