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
                            return RollChance::End(game.first_player_is_prot);
                        }
                        RollChance::Combine(
                            moves
                                .into_iter()
                                .map(|mov| {
                                    let Move::Continue { game } = mov else {
                                        unreachable!()
                                    };
                                    Var(state_indices[&game])
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
    pub fn converge(&mut self) -> f32 {
        let new_vals: Vec<_> = self
            .exprs
            .iter()
            .map(|expr| expr.eval(&self.vals))
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

        total_delta
    }
}

#[derive(Debug, Clone)]
pub struct Var(u32);
// c * max(var...7) + c * max(var...7) + c * max(var...7) + c * max(var...7) + c * max(var...7) + k

#[derive(Debug, Clone)]
pub enum RollChance {
    End(bool),
    Combine([Option<Var>; 7]),
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
                    RollChance::End(win) => {
                        if *win {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    RollChance::Combine(inverses) => inverses
                        .iter()
                        .filter_map(|x| x.clone())
                        .map(|Var(index)| vals[index as usize])
                        .fold(f32::NEG_INFINITY, f32::max),
                };
                Roll::from_index(i).weight() * chance / 16.0
            })
            .sum()
    }
}
