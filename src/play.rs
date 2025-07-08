use itertools::Itertools;

use crate::{
    game::{GameState, Move, PossibleMovesIter, Roll},
    input,
    render::render,
};

fn get_input<T>(prompt: &str, mut func: impl FnMut(String) -> Option<T>) -> T {
    loop {
        println!("{}", prompt);
        if let Some(res) = func(input().trim().to_string()) {
            return res;
        }
    }
}

pub fn play() {
    loop {
        let mut game = GameState::new();
        loop {
            println!("{}", render(&game));
            let roll = get_input("roll: ", |s| Roll::from_index(s.parse().ok()?));
            let moves = PossibleMovesIter::new(game, roll).collect_vec();
            println!("moves: {:?}", moves);
            let mov = get_input("move index: ", |s| moves.get(s.parse::<usize>().ok()?));
            match mov {
                Move::Continue { game: new_game, .. } => {
                    game = new_game.clone();
                }
                Move::End { .. } => {
                    println!("ended");
                    break;
                }
            }
        }
    }
}
