use std::io;

use crate::{play::play, render::render, solve::solve};

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
    // play();
    let (_, states, table) = solve();
    loop {
        println!("Choose state to view: ");
        let Ok(index) = input().parse::<usize>() else {
            continue;
        };
        if index > states.len() {
            continue;
        }
        let game = states[index];
        let val = table.vals[index];
        let expr = table.exprs[index].clone();
        println!("{}", render(&game));
        println!("val: {val}, expr: {expr:?}",);
    }
}
