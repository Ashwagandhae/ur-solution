// use rustc_hash::FxHashMap as HashMap;

// use rayon::prelude::*;

// use crate::{
//     game::GameState,
//     solve::expr::{eval_expr, get_exprs, ExprPart},
// };

// #[derive(Debug, Clone)]
// pub struct Table {
//     pub vals: Vec<f64>,
//     pub old_vals: Vec<f64>,
//     pub expr_parts: Vec<ExprPart>,
//     pub expr_starts: Vec<u32>,
// }

// impl Table {
//     pub fn new(state_indices: &HashMap<GameState, u32>, states: &[GameState]) -> Self {
//         let (expr_parts, expr_starts) = get_exprs(state_indices, states);
//         println!("number of expr parts: {}", expr_parts.len());

//         let vals = vec![0.0; states.len()];
//         let old_vals = vec![0.0; states.len()];

//         Self {
//             expr_parts,
//             expr_starts,
//             vals,
//             old_vals,
//         }
//     }
//     pub fn converge(&mut self) {
//         std::mem::swap(&mut self.vals, &mut self.old_vals);

//         self.vals.par_iter_mut().enumerate().for_each(|(i, val)| {
//             *val = eval_expr(i, &self.expr_parts, &self.expr_starts, &self.old_vals);
//         });

//         // let total_delta: f64 = self
//         //     .vals
//         //     .par_iter()
//         //     .zip(self.old_vals.par_iter())
//         //     .map(|(x, y)| (x - y).abs())
//         //     .sum();

//         // let max_delta: f64 = self
//         //     .vals
//         //     .par_iter()
//         //     .zip(self.old_vals.par_iter())
//         //     .map(|(x, y)| (x - y).abs())
//         //     .reduce(|| f64::NEG_INFINITY, f64::max);

//         // println!(
//         //     "total delta: {}, average delta: {}, max delta: {}",
//         //     total_delta,
//         //     total_delta / self.vals.len() as f64,
//         //     max_delta
//         // );

//         // total_delta
//     }
//     pub fn stats(&self) {
//         let total_delta: f64 = self
//             .vals
//             .par_iter()
//             .zip(self.old_vals.par_iter())
//             .map(|(x, y)| (x - y).abs())
//             .sum();

//         let max_delta: f64 = self
//             .vals
//             .par_iter()
//             .zip(self.old_vals.par_iter())
//             .map(|(x, y)| (x - y).abs())
//             .reduce(|| f64::NEG_INFINITY, f64::max);

//         println!(
//             "total delta: {}, average delta: {}, max delta: {}",
//             total_delta,
//             total_delta / self.vals.len() as f64,
//             max_delta
//         );
//     }

//     pub fn vals(&self) -> &Vec<f64> {
//         &self.vals
//     }
// }
