use rayon::prelude::*;

use crate::{
    game::GameStateSmall,
    solve::{
        converge_gpu::{Converger, DeviceHolder},
        expr::{create_exprs, eval_expr, ExprPart},
    },
};

pub const THRESHOLD_DELTA: f32 = 1e-6;
pub const GPU_THRESHOLD: usize = 50_000;

pub fn converge(
    states: &[GameStateSmall],
    vals: &mut [f32],
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
        converge_gpu(
            dep_start,
            dep_vals,
            vals,
            &expr_parts,
            &expr_starts,
            device_holder,
        );
    } else {
        converge_cpu(dep_start, dep_vals, vals, &expr_parts, &expr_starts);
    }
}

fn converge_gpu(
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
        if delta <= THRESHOLD_DELTA {
            println!("final delta {} after {} iters", delta, iters);
            break;
        }
    }

    vals.copy_from_slice(out_vals);
}

fn converge_cpu(
    dep_start: usize,
    dep_vals: &[f32],
    vals: &mut [f32],
    expr_parts: &[ExprPart],
    expr_starts: &[u32],
) {
    let mut out_vals: &mut [f32] = &mut vals.to_vec();
    let mut in_vals = vals;
    let mut iters = 0;
    let mut swapped = false;
    loop {
        iters += 1;
        step(
            dep_start,
            &dep_vals,
            in_vals,
            out_vals,
            &expr_parts,
            &expr_starts,
        );

        let delta = max_delta(in_vals, out_vals);

        if delta <= THRESHOLD_DELTA {
            println!("final delta {} after {} iters", delta, iters);
            break;
        }

        (in_vals, out_vals) = (out_vals, in_vals);
        swapped = !swapped;
    }
    if !swapped {
        // we know vals doesn't have the most recent values, so we need to update
        let vals = in_vals;
        vals.copy_from_slice(&out_vals);
    }
}

pub fn step(
    dep_start: usize,

    dep_vals: &[f32],
    in_vals: &[f32],
    out_vals: &mut [f32],

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
        *val = eval_expr(&expr_parts, expr_starts[i] as usize, get_val);
    });
}

pub fn max_delta(vals: &[f32], old_vals: &[f32]) -> f32 {
    vals.par_iter()
        .zip(old_vals.par_iter())
        .map(|(val, old_val)| (val - old_val).abs())
        .reduce(|| 0.0, f32::max)
}
