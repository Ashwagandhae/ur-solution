use core::f64;
use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    game::{GameState, GameStateSmall, Move, PossibleMovesIter, Roll, GOAL_SCORE},
    save::read_or_create,
    successor::Succ,
};
use num_traits::Float;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// pub fn create_exprs(
//     states: &[GameStateSmall],
//     dep_start: usize,
//     start: usize,
//     end: usize,
//     expr_parts: &mut Vec<ExprPart>,
//     expr_starts: &mut Vec<u32>,
// ) {
//     println!("create exprs normal");
//     for game in &states[start..end] {
//         expr_starts.push(expr_parts.len().try_into().expect("expr index too big"));
//         for roll in Roll::succ_iter() {
//             let first_part_i = expr_parts.len(); // remember where this roll starts

//             println!(
//                 "iter len: {}",
//                 PossibleMovesIter::new(GameState::from(*game), roll).count()
//             );
//             for mov in PossibleMovesIter::new(GameState::from(*game), roll) {
//                 println!("got mov: {:?}", mov);
//                 match mov {
//                     Move::End => {
//                         expr_parts.truncate(first_part_i);
//                         expr_parts.push(ExprPart::new(true, false, Val::Win));
//                         break;
//                     }
//                     Move::Continue { game, keep_turn } => {
//                         let game = if keep_turn { game } else { game.flipped() };
//                         let idx: u32 = ((&states[dep_start..end])
//                             .binary_search(&GameStateSmall::from(game))
//                             .unwrap()
//                             + dep_start)
//                             .try_into()
//                             .expect("too many game states for u32");
//                         expr_parts.push(ExprPart::new(false, !keep_turn, Val::Var(idx)));
//                     }
//                 }
//             }

//             let last = expr_parts.last_mut().unwrap();
//             *last = ExprPart::new(true, last.is_inverse(), last.get_val());
//         }
//     }
// }

pub fn create_exprs(
    states: &[GameStateSmall],
    dep_start: usize,
    start: usize,
    end: usize,
    expr_parts: &mut Vec<ExprPart>,
    expr_starts: &mut Vec<u32>,
) {
    let parts: Vec<_> = (&states[start..end])
        .par_iter()
        .flat_map_iter(|game| {
            Roll::succ_iter().flat_map(|roll| {
                let mut buf: [Option<ExprPart>; 7] = [None; 7];
                let mut index = 0;

                for mov in PossibleMovesIter::new(GameState::from(*game), roll) {
                    match mov {
                        Move::End => {
                            buf[0] = Some(ExprPart::new(true, false, Val::Win));
                            buf[1] = None;

                            return buf.into_iter().while_some();
                        }
                        Move::Continue { game, keep_turn } => {
                            let game = if keep_turn { game } else { game.flipped() };
                            let idx: u32 = ((&states[dep_start..end])
                                .binary_search(&GameStateSmall::from(game))
                                .unwrap()
                                + dep_start)
                                .try_into()
                                .expect("too many game states for u32");
                            buf[index] = Some(ExprPart::new(false, !keep_turn, Val::Var(idx)));
                            index += 1;
                        }
                    }
                }
                let last = &mut buf[index - 1];
                *last = Some(ExprPart::new(
                    true,
                    last.unwrap().is_inverse(),
                    last.unwrap().get_val(),
                ));
                buf.into_iter().while_some()
            })
        })
        .collect();
    expr_parts.extend(parts);
    let mut roll = 0;
    for (i, part) in expr_parts.iter().enumerate() {
        if roll % Roll::vals().len() == 0 {
            expr_starts.push(i as u32);
        }
        if part.is_end() {
            roll += 1;
        }
    }
}

pub fn eval_expr<T: Float>(
    expr_parts: &[ExprPart],
    first_part_index: usize,
    get_val: impl Fn(usize) -> T,
) -> T {
    let mut i = first_part_index;
    let mut current_roll = Some(Roll::first());
    let mut sum: T = T::zero();
    let mut current_max: T = T::neg_infinity();
    while let Some(roll) = current_roll {
        let part = &expr_parts[i];

        let val = match part.get_val() {
            Val::Win => T::one(),
            Val::Var(index) => get_val(index as usize),
        };
        let val = if part.is_inverse() {
            T::one() - val
        } else {
            val
        };

        current_max = current_max.max(val);

        if part.is_end() {
            sum = sum + roll.weight::<T>() * current_max;

            current_max = T::neg_infinity();
            current_roll = roll.succ();
        }
        i += 1;
    }
    sum
}
