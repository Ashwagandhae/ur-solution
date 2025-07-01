use std::collections::HashMap;

use itertools::Itertools;

use crate::game::{possible_moves, GameState, Move, Roll};

#[derive(Debug, Clone)]
pub struct Table {
    pub vals: Vec<f32>,
    pub exprs: Vec<Expr>,
}

impl Table {
    pub fn new(state_indices: &HashMap<GameState, u32>, states: &[GameState]) -> Self {
        let exprs = states
            .iter()
            .map(|game| {
                let roll_chances: [RollChance; 5] = Roll::iter_all()
                    .map(|roll| {
                        let moves = possible_moves(*game, roll);
                        if let Some(_) = moves.iter().find(|m| matches!(m, Move::End { .. })) {
                            return RollChance::Win;
                        }
                        RollChance::Combine(
                            moves
                                .into_iter()
                                .filter_map(|mov| {
                                    if let Move::Continue { game, keep_turn } = mov {
                                        Some(if keep_turn {
                                            MaybeInverse::NoInverse(Var(state_indices[&game]))
                                        } else {
                                            MaybeInverse::Inverse(Var(
                                                state_indices[&game.flipped()]
                                            ))
                                        })
                                    } else {
                                        None
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
            .collect();

        let vals = vec![0.0; states.len()];

        Self { exprs, vals }
    }
    pub fn converge(&mut self) {
        let new_vals: Vec<_> = self
            .exprs
            .iter()
            .map(|expr| expr.eval(&self.vals))
            .map(|val| val.clamp(0.0, 1.0))
            .collect();
        let total_delta: f32 = new_vals
            .iter()
            .zip(self.vals.iter())
            .map(|(x, y)| (x - y).abs())
            .sum();
        println!(
            "total delta: {}, average delta: {}",
            total_delta,
            total_delta / self.vals.len() as f32
        );

        let damping = 0.5;
        for (x, &new_x) in self.vals.iter_mut().zip(new_vals.iter()) {
            *x = *x * (1.0 - damping) + new_x * damping;
        }

        self.vals = new_vals;
    }
}

#[derive(Debug, Clone)]
pub enum MaybeInverse {
    Inverse(Var),
    NoInverse(Var),
}

#[derive(Debug, Clone)]
pub struct Var(u32);
// c * max(var...7) + c * max(var...7) + c * max(var...7) + c * max(var...7) + c * max(var...7) + k

#[derive(Debug, Clone)]
pub enum RollChance {
    Win,
    Combine([Option<MaybeInverse>; 7]),
}

#[derive(Debug, Clone)]
pub struct Expr {
    roll_chances: [RollChance; 5],
}

impl Expr {
    fn eval(&self, vals: &[f32]) -> f32 {
        self.roll_chances
            .iter()
            .enumerate()
            .map(|(i, roll_chance)| {
                let chance = match roll_chance {
                    RollChance::Win => 1.0,
                    RollChance::Combine(inverses) => inverses
                        .iter()
                        .filter_map(|x| x.clone())
                        .map(|inverse| match inverse {
                            MaybeInverse::Inverse(Var(index)) => 1.0 - vals[index as usize],
                            MaybeInverse::NoInverse(Var(index)) => vals[index as usize],
                        })
                        .fold(f32::NEG_INFINITY, f32::max),
                };
                Roll::from_index(i).weight() * chance
            })
            .sum()
    }
}
