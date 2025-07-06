use std::io;

use crate::{
    render::render,
    solve::{
        expr::{create_exprs, RollChance},
        get_mappings, solve,
        table_gpu::test,
    },
};

mod game;
mod play;
mod render;
mod solve;

pub fn input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}
fn main() {
    env_logger::init();

    for _ in 0..10 {
        test();
    }
    // // play();
    // let (_, states, table) = solve();
    // loop {
    //     println!("Choose state to view: ");
    //     let Ok(index) = input().parse::<usize>() else {
    //         continue;
    //     };
    //     if index > states.len() {
    //         continue;
    //     }
    //     let game = states[index];
    //     let val = table.vals[index];
    //     let expr = table.exprs[index].clone();
    //     println!("{}", render(&game));
    //     println!("val: {val}, expr: {expr:?}",);
    // }
}
