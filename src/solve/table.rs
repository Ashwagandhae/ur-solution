use std::collections::HashMap;

use rayon::prelude::*;

use crate::{
    game::GameState,
    solve::expr::{create_exprs, Expr},
};

#[derive(Debug, Clone)]
pub struct Table {
    pub vals: Vec<f64>,
    pub exprs: Vec<Expr>,
}

impl Table {
    pub fn new(state_indices: &HashMap<GameState, u32>, states: &[GameState]) -> Self {
        let exprs = create_exprs(state_indices, states);

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
