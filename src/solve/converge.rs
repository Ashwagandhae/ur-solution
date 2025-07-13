use std::fmt::Display;

use num_traits::Float;
use rayon::prelude::*;

use crate::{
    game::GameStateSmall,
    solve::{
        converge_gpu::{Converger, DeviceHolder},
        expr::{create_exprs, eval_expr, ExprPart},
    },
};

pub const THRESHOLD_DELTA_32: f32 = 1e-6;
pub const THRESHOLD_DELTA_64: f64 = 1e-15;
pub const GPU_THRESHOLD: usize = 100_000_000;
pub const MAX_ITERS: usize = 2000;

pub fn converge(
    states: &[GameStateSmall],
    vals: &mut [f64],
    dep_start: usize,
    start: usize,
    end: usize,
    device_holder: &mut DeviceHolder,
    expr_parts: &mut Vec<ExprPart>,
    expr_starts: &mut Vec<u32>,
) {
    println!(
        "converge vals {}..{} ({}, deps {}) out of {}",
        start,
        end,
        end - start,
        start - dep_start,
        states.len()
    );

    expr_parts.clear();
    expr_starts.clear();

    create_exprs(states, dep_start, start, end, expr_parts, expr_starts);

    let [dep_vals, vals] = vals
        .get_disjoint_mut([dep_start..start, start..end])
        .unwrap();

    // set vals to non nan
    for val in vals.iter_mut() {
        *val = 0.0;
    }

    if vals.len() > GPU_THRESHOLD {
        converge_gpu_f64(
            dep_start,
            dep_vals,
            vals,
            expr_parts,
            expr_starts,
            device_holder,
        );
    } else {
        converge_cpu(
            dep_start,
            dep_vals,
            vals,
            expr_parts,
            expr_starts,
            THRESHOLD_DELTA_64,
        );
    }
}

fn converge_gpu_f32(
    dep_start: usize,
    dep_vals: &[f32],
    vals: &mut [f32],
    expr_parts: &[ExprPart],
    expr_starts: &[u32],
    device_holder: &mut DeviceHolder,
) {
    println!("using gpu...");
    let mut converger = Converger::new(
        device_holder,
        dep_start,
        dep_vals,
        vals.len(),
        expr_parts,
        expr_starts,
    );
    let in_vals = &mut vec![0.0; vals.len()];
    let out_vals = &mut vec![0.0; vals.len()];

    let mut iters = 0;
    loop {
        iters += 20;
        converger.converge(device_holder, 20, in_vals, out_vals);
        let delta = max_delta(in_vals, out_vals);
        if delta <= THRESHOLD_DELTA_32 {
            println!("final delta {delta} after {iters} iters");
            break;
        }
        if iters > MAX_ITERS {
            panic!("reached max iters: {MAX_ITERS}");
            break;
        }
    }

    vals.copy_from_slice(out_vals);
}
fn converge_gpu_f64(
    dep_start: usize,
    dep_vals: &[f64],
    vals: &mut [f64],
    expr_parts: &[ExprPart],
    expr_starts: &[u32],
    device_holder: &mut DeviceHolder,
) {
    println!("using gpu...");
    let dep_vals_f32: Vec<_> = dep_vals.iter().map(|f| *f as f32).collect();
    let mut converger = Converger::new(
        device_holder,
        dep_start,
        &dep_vals_f32,
        vals.len(),
        expr_parts,
        expr_starts,
    );
    let in_vals = &mut vec![0.0; vals.len()];
    let out_vals = &mut vec![0.0; vals.len()];

    let mut iters = 0;
    loop {
        iters += 20;
        converger.converge(device_holder, 20, in_vals, out_vals);
        let delta = max_delta(in_vals, out_vals);
        if delta <= THRESHOLD_DELTA_32 {
            println!("switching to cpu after {iters} iters");
            break;
        }
        if iters > MAX_ITERS {
            println!("reached max iters: {MAX_ITERS}");
            break;
        }
    }

    for i in 0..vals.len() {
        vals[i] = out_vals[i] as f64;
    }

    converge_cpu(
        dep_start,
        dep_vals,
        vals,
        expr_parts,
        expr_starts,
        THRESHOLD_DELTA_64,
    );
}

fn converge_cpu<T: Float + Send + Sync + Display>(
    dep_start: usize,
    dep_vals: &[T],
    vals: &mut [T],
    expr_parts: &[ExprPart],
    expr_starts: &[u32],
    threshold_delta: T,
) {
    let mut out_vals: &mut [T] = &mut vals.to_vec();
    let mut in_vals = vals;
    let mut iters = 0;
    let mut swapped = false;
    loop {
        iters += 1;
        step(
            dep_start,
            dep_vals,
            in_vals,
            out_vals,
            expr_parts,
            expr_starts,
        );

        let delta = max_delta(in_vals, out_vals);

        if delta <= threshold_delta {
            println!("final delta {delta} after {iters} iters");
            break;
        }
        if iters > MAX_ITERS {
            panic!("reached max iters: {MAX_ITERS}");
            break;
        }

        (in_vals, out_vals) = (out_vals, in_vals);
        swapped = !swapped;
    }
    if !swapped {
        // we know vals doesn't have the most recent values, so we need to update
        let vals = in_vals;
        vals.copy_from_slice(out_vals);
    }
}

pub fn step<T: Float + Send + Sync>(
    dep_start: usize,

    dep_vals: &[T],
    in_vals: &[T],
    out_vals: &mut [T],

    expr_parts: &[ExprPart],
    expr_starts: &[u32],
) {
    let get_val = |i| {
        let dep_index = i - dep_start;
        if dep_index < dep_vals.len() {
            dep_vals[dep_index]
        } else {
            in_vals[dep_index - dep_vals.len()]
        }
    };
    out_vals.par_iter_mut().enumerate().for_each(|(i, val)| {
        *val = eval_expr(expr_parts, expr_starts[i] as usize, get_val);
    });
}

pub fn max_delta<T: Float + Send + Sync>(vals: &[T], old_vals: &[T]) -> T {
    vals.par_iter()
        .zip(old_vals.par_iter())
        .map(|(val, old_val)| (*val - *old_val).abs())
        .reduce(|| T::zero(), T::max)
}
