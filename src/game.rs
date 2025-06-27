use crate::game::strip::{Delta, DeltaResult, MoveSource, Square, StripIndex, StripState};

pub mod strip;

pub const GOAL_SCORE: u8 = 3;

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
                    Move::End {
                        prot: game.prot,
                        opp: game.opp,
                    }
                } else {
                    Move::Continue {
                        game,
                        caused_deletion: false,
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
                        caused_deletion,
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
    Continue {
        game: GameState,
        keep_turn: bool,
        caused_deletion: bool,
    },
    End {
        prot: TeamState,
        opp: TeamState,
    },
}
