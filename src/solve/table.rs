use std::collections::HashMap;

use itertools::Itertools;
use rayon::prelude::*;

use crate::game::{possible_moves, GameState, Move, Roll};

#[derive(Debug, Clone)]
pub struct Table {
    pub vals: Vec<f64>,
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
            .collect();

        let vals = vec![0.0; states.len()];

        Self { exprs, vals }
    }
    pub fn converge(&mut self) -> f64 {
        let mut new_vals: Vec<_> = self
            .exprs
            .par_iter()
            .map(|expr| expr.eval(&self.vals))
            .collect();
        let total_delta: f64 = new_vals
            .par_iter()
            .zip(self.vals.par_iter())
            .map(|(x, y)| (x - y).abs())
            .sum();

        println!(
            "total delta: {}, average delta: {}",
            total_delta,
            total_delta / self.vals.len() as f64
        );
        new_vals[0] = new_vals[0].clamp(0.3, 0.7);

        let damping = 0.8;
        for (x, &new_x) in self.vals.iter_mut().zip(new_vals.iter()) {
            *x = *x * (1.0 - damping) + new_x * damping;
        }
        self.vals = new_vals;

        total_delta
    }
}

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
    roll_chances: [RollChance; 5],
}

impl Expr {
    fn eval(&self, vals: &[f64]) -> f64 {
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
