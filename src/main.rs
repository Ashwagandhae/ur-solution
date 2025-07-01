use std::io;

use crate::solve::solve;

mod game;
mod play;
mod render;
mod solve;

fn input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

fn main() {
    solve();
}
