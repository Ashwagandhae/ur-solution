use itertools::Itertools;
use std::collections::HashMap;

use crate::{
    game::{
        strip::{Delta, MoveSource},
        GameState, Move,
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
                .filter_map(|Roll { delta, weight }| {
                    let mut possible_moves: Vec<_> = match delta {
                        None => vec![Move::Continue {
                            game: game.clone(),
                            keep_turn: false,
                            caused_deletion: false,
                        }],
                        Some(delta) => MoveSource::iter_all()
                            .filter_map(|source| game.move_piece(source, delta))
                            .collect(),
                    };

                    if possible_moves.is_empty() {
                        // skip turn
                        possible_moves = vec![Move::Continue {
                            game: game.clone(),
                            keep_turn: false,
                            caused_deletion: false,
                        }];
                    }
                    // println!("possible moves for roll {:?} was:", delta,);
                    // for mov in possible_moves.iter() {
                    //     println!("===mov===");
                    //     match mov {
                    //         Move::Continue {
                    //             game: other_game,
                    //             keep_turn,
                    //             ..
                    //         } => {
                    //             println!(
                    //                 "keep_turn: {keep_turn}, game_eq: {}",
                    //                 *other_game == game
                    //             );
                    //             println!("{}", render(other_game));
                    //             println!(
                    //                 "current status of game in hashmap: {:?}",
                    //                 (if *keep_turn {
                    //                     memo.get(other_game)
                    //                 } else {
                    //                     memo.get(&other_game.flipped())
                    //                 })
                    //             )
                    //         }
                    //         Move::End { prot, opp } => {
                    //             println!("end")
                    //         }
                    //     }
                    // }

                    let best_chance = possible_moves
                        .into_iter()
                        .sorted_by_key(|mov| match mov {
                            Move::End { .. } => 0,
                            Move::Continue {
                                caused_deletion: false,
                                ..
                            } => 1,
                            Move::Continue {
                                caused_deletion: true,
                                ..
                            } => 2,
                        })
                        .filter(|mov| {
                            !matches!(
                                mov,
                                Move::Continue {
                                    caused_deletion: true,
                                    ..
                                }
                            )
                        })
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
                    best_chance.map(|chance| (chance, weight))
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

pub struct Roll {
    delta: Option<Delta>,
    weight: f32,
}

impl Roll {
    fn iter_all() -> impl Iterator<Item = Roll> {
        [
            Roll {
                delta: None,
                weight: 1.0,
            },
            Roll {
                delta: Delta::new(1),
                weight: 4.0,
            },
            Roll {
                delta: Delta::new(2),
                weight: 6.0,
            },
            Roll {
                delta: Delta::new(3),
                weight: 4.0,
            },
            Roll {
                delta: Delta::new(4),
                weight: 1.0,
            },
        ]
        .into_iter()
    }
}
