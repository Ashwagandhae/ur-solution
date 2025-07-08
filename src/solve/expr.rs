use core::f64;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    game::{GameState, Move, PossibleMovesIter, Roll, GOAL_SCORE},
    save::read_or_create,
    successor::Succ,
};

#[derive(Debug, Clone, Copy)]
pub struct ExprPart(u32);

impl ExprPart {
    const LOWER_29_MASK: u32 = 0b0001_1111_1111_1111_1111_1111_1111_1111;
    fn new(end: bool, inverse: bool, val: Val) -> Self {
        let mut bits: u32 = 0;
        if end {
            bits |= 1 << 31;
        }
        if inverse {
            bits |= 1 << 30;
        }
        match val {
            Val::Win => {
                bits |= 1 << 29;
            }
            Val::Var(index) => {
                let lower_29 = Self::LOWER_29_MASK & index;
                bits |= lower_29;
            }
        }
        Self(bits)
    }

    fn is_end(&self) -> bool {
        (self.0 & (1 << 31)) != 0
    }

    fn is_inverse(&self) -> bool {
        (self.0 & (1 << 30)) != 0
    }

    fn get_val(&self) -> Val {
        let is_win = (self.0 & (1 << 29)) != 0;
        if is_win {
            Val::Win
        } else {
            Val::Var(self.0 & Self::LOWER_29_MASK)
        }
    }
}

impl From<ExprPart> for u32 {
    fn from(part: ExprPart) -> u32 {
        part.0
    }
}
impl From<u32> for ExprPart {
    fn from(value: u32) -> ExprPart {
        ExprPart(value)
    }
}

#[derive(Debug, Clone)]
pub enum Val {
    Win,
    Var(u32),
}

pub fn get_exprs(
    state_indices: &HashMap<GameState, u32>,
    states: &[GameState],
) -> (Vec<ExprPart>, Vec<u32>) {
    read_or_create(
        &format!("./data/exprs_{}.bin", GOAL_SCORE),
        || create_exprs(state_indices, states),
        |(expr_parts, expr_starts)| {
            (
                expr_parts.iter().cloned().map(u32::from).collect(),
                expr_starts.clone(),
            )
        },
        |(parts, expr_starts): &(Vec<_>, _)| {
            (
                parts.iter().cloned().map(ExprPart::from).collect(),
                expr_starts.clone(),
            )
        },
    )
}

fn create_exprs(
    state_indices: &HashMap<GameState, u32>,
    states: &[GameState],
) -> (Vec<ExprPart>, Vec<u32>) {
    let mut expr_parts = Vec::new();
    let mut expr_starts = Vec::new();
    for (i, game) in states.iter().enumerate() {
        if i % 1_000_000 == 0 {
            println!("created {} expr parts for {} states", expr_parts.len(), i);
        }
        expr_starts.push(expr_parts.len().try_into().expect("expr index too big"));
        for roll in Roll::succ_iter() {
            let first_part_i = expr_parts.len(); // remember where this roll starts

            for mov in PossibleMovesIter::new(*game, roll) {
                match mov {
                    Move::End => {
                        expr_parts.truncate(first_part_i);
                        expr_parts.push(ExprPart::new(true, false, Val::Win));
                        break;
                    }
                    Move::Continue { game, keep_turn } => {
                        let idx = state_indices[&if keep_turn { game } else { game.flipped() }];
                        expr_parts.push(ExprPart::new(false, !keep_turn, Val::Var(idx)));
                    }
                }
            }

            let last = expr_parts.last_mut().unwrap();
            *last = ExprPart::new(true, last.is_inverse(), last.get_val());
        }
    }
    (expr_parts, expr_starts)
}

pub fn eval_expr(
    expr_index: usize,
    expr_parts: &[ExprPart],
    expr_starts: &[u32],
    vals: &[f64],
) -> f64 {
    let mut i = expr_starts[expr_index] as usize;
    let mut current_roll = Some(Roll::first());
    let mut sum: f64 = 0.0;
    let mut current_max: f64 = f64::NEG_INFINITY;
    while let Some(roll) = current_roll {
        let part = &expr_parts[i];

        let val = match part.get_val() {
            Val::Win => 1.0,
            Val::Var(index) => vals[index as usize],
        };
        let val = if part.is_inverse() { 1.0 - val } else { val };

        current_max = current_max.max(val);

        if part.is_end() {
            sum += roll.weight() * current_max;

            current_max = f64::NEG_INFINITY;
            current_roll = roll.succ();
        }
        i += 1;
    }
    sum
}
