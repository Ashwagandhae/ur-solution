use std::io;

use crate::{game::GameState, play::play};

mod game;
mod play;
mod render;

fn input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

fn main() {
    println!("Hello, world!");
    // let test = vec![0.0; 753_831_936];
    // for (i, x) in test.iter().enumerate() {
    //     println!("{}, {}", i, x);
    // }
    let res = play();
    println!(
        "chance of winning at the start: {:?}",
        res.get(&GameState::new()).unwrap()
    )
}
