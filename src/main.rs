use std::io;

use crate::{game::GameState, render::render, solve::solve};

mod game;
mod play;
mod render;
mod save;
mod solve;
mod successor;

pub fn input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}
fn main() {
    env_logger::init();

    let (states, vals) = solve();
    loop {
        println!("Choose state to view: ");
        let Ok(index) = input().parse::<usize>() else {
            continue;
        };
        if index > states.len() {
            continue;
        }
        let game = states[index];
        let val = vals[index];
        println!("{}", render(&GameState::from(game)));
        println!("val: {val}",);
    }
}
