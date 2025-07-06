use crate::game::strip::{Delta, DeltaResult, MoveSource, Square, StripIndex, StripState};

pub mod strip;

pub const GOAL_SCORE: u8 = 7;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct GameState {
    pub prot: TeamState,
    pub opp: TeamState,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            prot: TeamState::new(),
            opp: TeamState::new(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct TeamState {
    pub strip: StripState,
    pub score: u8,
}

impl TeamState {
    pub fn new() -> Self {
        Self {
            strip: StripState::new(),
            score: 0,
        }
    }
}

impl TeamState {
    fn remaining(&self) -> u8 {
        GOAL_SCORE - self.strip.count_pieces() - self.score
    }

    fn remove_move_source(&self, source: MoveSource) -> Option<Self> {
        match source {
            MoveSource::Index(i) => {
                if self.strip.get(i) {
                    let mut state = self.clone();
                    state.strip.set(i, false);
                    Some(state)
                } else {
                    None
                }
            }
            MoveSource::Launch => {
                if self.remaining() == 0 {
                    None
                } else {
                    Some(self.clone())
                }
            }
        }
    }
}

impl GameState {
    pub fn flipped(self) -> GameState {
        GameState {
            prot: self.opp,
            opp: self.prot,
        }
    }

    pub fn player_at_i(&self, i: StripIndex) -> Option<Player> {
        if i.both_teams_accessible() {
            match (self.prot.strip.get(i), self.opp.strip.get(i)) {
                (true, false) => Some(Player::Prot),
                (false, true) => Some(Player::Opp),
                (false, false) => None,
                _ => panic!("prot and opp strips both have piece at {i:?}"),
            }
        } else {
            if self.prot.strip.get(i) {
                Some(Player::Prot)
            } else {
                None
            }
        }
    }

    pub fn move_piece(&self, source: MoveSource, delta: Delta) -> Option<Move> {
        let mut game = self.clone();
        game.prot = game.prot.remove_move_source(source)?;
        match source.apply_delta(delta) {
            DeltaResult::OutOfBounds => None,
            DeltaResult::Score => {
                game.prot.score += 1;
                Some(if game.prot.score == GOAL_SCORE {
                    Move::End
                } else {
                    Move::Continue {
                        game,
                        keep_turn: false,
                    }
                })
            }
            DeltaResult::Index(new_i) => match (self.player_at_i(new_i), new_i.square()) {
                (Some(Player::Prot), _) => None,
                (Some(Player::Opp), Square::Flower) => None,
                (opp, square) => {
                    let caused_deletion = matches!(opp, Some(Player::Opp));
                    if caused_deletion {
                        game.opp.strip.set(new_i, false);
                    }
                    game.prot.strip.set(new_i, true);
                    Some(Move::Continue {
                        game,
                        keep_turn: matches!(square, Square::Flower),
                    })
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Player {
    Prot,
    Opp,
}

#[derive(Debug, Clone)]
pub enum Move {
    Continue { game: GameState, keep_turn: bool },
    End,
}

#[derive(Debug, Clone, Copy)]
pub enum Roll {
    Zero,
    Delta(Delta),
}
impl Roll {
    pub fn vals() -> [Roll; 5] {
        [
            Roll::Zero,
            Roll::Delta(Delta::new(1).unwrap()),
            Roll::Delta(Delta::new(2).unwrap()),
            Roll::Delta(Delta::new(3).unwrap()),
            Roll::Delta(Delta::new(4).unwrap()),
        ]
    }
    pub fn iter_all() -> impl Iterator<Item = Roll> {
        Self::vals().into_iter()
    }

    pub fn from_index(index: usize) -> Option<Self> {
        Self::vals().get(index).cloned()
    }

    pub fn weight(&self) -> f64 {
        match self {
            Roll::Zero => 1.0,
            Roll::Delta(delta) if delta.get() == 1 => 4.0,
            Roll::Delta(delta) if delta.get() == 2 => 6.0,
            Roll::Delta(delta) if delta.get() == 3 => 4.0,
            Roll::Delta(delta) if delta.get() == 4 => 1.0,
            _ => unreachable!(),
        }
    }
}

pub fn possible_moves(game: GameState, roll: Roll) -> Vec<Move> {
    let mut possible_moves: Vec<_> = match roll {
        Roll::Zero => vec![Move::Continue {
            game: game.clone(),
            keep_turn: false,
        }],
        Roll::Delta(delta) => MoveSource::iter_all()
            .filter_map(|source| game.move_piece(source, delta))
            .collect(),
    };

    if possible_moves.is_empty() {
        // skip turn
        possible_moves = vec![Move::Continue {
            game: game.clone(),
            keep_turn: false,
        }];
    }
    possible_moves
}
