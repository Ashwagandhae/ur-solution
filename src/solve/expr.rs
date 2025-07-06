use itertools::Itertools;
use std::collections::HashMap;

use crate::game::{possible_moves, GameState, Move, Roll};

#[derive(Debug, Clone)]
pub struct Var(u32);

#[derive(Debug, Clone)]
pub struct MaybeInverse {
    inverse: bool,
    var: Var,
}

#[derive(Debug, Clone)]
pub enum RollChance {
    End,
    Combine([Option<MaybeInverse>; 7]),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub roll_chances: [RollChance; 5],
}

impl Expr {
    pub fn eval(&self, vals: &[f64]) -> f64 {
        self.roll_chances
            .iter()
            .enumerate()
            .map(|(i, roll_chance)| {
                let chance = match roll_chance {
                    RollChance::End => 1.0,
                    RollChance::Combine(inverses) => inverses
                        .iter()
                        .filter_map(|x| x.clone())
                        .map(|maybe_inverse| {
                            let val = vals[maybe_inverse.var.0 as usize];
                            if maybe_inverse.inverse {
                                1.0 - val
                            } else {
                                val
                            }
                        })
                        .fold(f64::NEG_INFINITY, f64::max),
                };
                Roll::from_index(i).unwrap().weight() * chance / 16.0
            })
            .sum()
    }
}

pub fn create_exprs(state_indices: &HashMap<GameState, u32>, states: &[GameState]) -> Vec<Expr> {
    states
        .iter()
        .map(|game| {
            let roll_chances: [RollChance; 5] = Roll::iter_all()
                .map(|roll| {
                    let moves = possible_moves(*game, roll);
                    if let Some(_) = moves.iter().find(|m| matches!(m, Move::End { .. })) {
                        return RollChance::End;
                    }
                    RollChance::Combine(
                        moves
                            .into_iter()
                            .map(|mov| {
                                let Move::Continue { game, keep_turn } = mov else {
                                    unreachable!()
                                };
                                MaybeInverse {
                                    inverse: !keep_turn,
                                    var: Var(state_indices
                                        [&if keep_turn { game } else { game.flipped() }]),
                                }
                            })
                            .map(Some)
                            .chain(std::iter::repeat(None))
                            .take(7)
                            .collect_array()
                            .unwrap(),
                    )
                })
                .collect_array()
                .unwrap();
            Expr { roll_chances }
        })
        .collect()
}
