use crate::{
    game::strip::{Delta, DeltaResult, MoveSource, Square, StripIndex, StripState},
    solve::perma::PermaKey,
    successor::{Succ, SuccIter},
};

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
// [3:score1][3:score2][4:start1][2:end1][4:start2][2:end2][13:shared]
// 30..=28   27..=25   24..=21   20..=19 18..=15   14..=13    12..=0
pub struct GameStateSmall(u32);

impl PartialOrd for GameStateSmall {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for GameStateSmall {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        PermaKey::new(GameState::from(*self))
            .cmp(&PermaKey::new(GameState::from(*other)))
            .then_with(|| self.0.cmp(&other.0))
    }
}
impl From<GameState> for GameStateSmall {
    fn from(value: GameState) -> Self {
        let mut res = 0;
        res |= ((value.prot.score & 0b0000_0111) as u32) << 28;
        res |= ((value.opp.score & 0b0000_0111) as u32) << 25;

        res |= (value.prot.strip.start_bits() as u32) << 21;
        res |= (value.prot.strip.end_bits() as u32) << 19;

        res |= (value.opp.strip.start_bits() as u32) << 15;
        res |= (value.opp.strip.end_bits() as u32) << 13;

        let shared = StripIndex::succ_iter()
            .skip(4)
            .take(8)
            .fold(0u32, |acc, i| {
                acc * 3
                    + match (value.prot.strip.get(i), value.opp.strip.get(i)) {
                        (false, false) => 0,
                        (true, false) => 1,
                        (false, true) => 2,
                        _ => panic!("prot and opp overlap"),
                    }
            });
        res |= 0b0111_1111_1111_1111 & shared;
        Self(res)
    }
}

impl From<GameStateSmall> for GameState {
    fn from(small: GameStateSmall) -> Self {
        let bits = small.0;

        let score1 = ((bits >> 28) & 0b111) as u8;
        let score2 = ((bits >> 25) & 0b111) as u8;

        let start1 = ((bits >> 21) & 0b1111) as u8;
        let end1 = ((bits >> 19) & 0b11) as u8;

        let start2 = ((bits >> 15) & 0b1111) as u8;
        let end2 = ((bits >> 13) & 0b11) as u8;

        let shared_encoded = bits & 0b01_1111_1111_1111;

        let mut prot_strip = StripState::from_start_and_end(start1, end1);
        let mut opp_strip = StripState::from_start_and_end(start2, end2);

        let mut shared = shared_encoded;
        for i in (4..12).rev() {
            let digit = shared % 3;
            shared /= 3;

            match digit {
                0 => {}
                1 => {
                    prot_strip.set(StripIndex::new(i).unwrap(), true);
                }
                2 => {
                    opp_strip.set(StripIndex::new(i).unwrap(), true);
                }
                _ => unreachable!(),
            }
        }

        GameState {
            prot: TeamState {
                score: score1,
                strip: prot_strip,
            },
            opp: TeamState {
                score: score2,
                strip: opp_strip,
            },
        }
    }
}

impl From<u32> for GameStateSmall {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<GameStateSmall> for u32 {
    fn from(value: GameStateSmall) -> Self {
        value.0
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn from_index(index: usize) -> Option<Self> {
        Self::vals().get(index).cloned()
    }

    pub fn weight(&self) -> f32 {
        match self {
            Roll::Zero => 1.0 / 16.0,
            Roll::Delta(delta) if delta.get() == 1 => 4.0 / 16.0,
            Roll::Delta(delta) if delta.get() == 2 => 6.0 / 16.0,
            Roll::Delta(delta) if delta.get() == 3 => 4.0 / 16.0,
            Roll::Delta(delta) if delta.get() == 4 => 1.0 / 16.0,
            _ => unreachable!(),
        }
    }
}

impl Succ for Roll {
    fn first() -> Self {
        Self::Zero
    }
    fn succ(&self) -> Option<Self> {
        Some(match self {
            Self::Zero => Self::Delta(Delta::first()),
            Self::Delta(d) => Self::Delta(d.succ()?),
        })
    }
}
#[derive(Debug, Clone)]
pub struct PossibleMovesIter {
    game: GameState,
    roll: PossibleMovesRoll,
}
#[derive(Debug, Clone)]
pub enum PossibleMovesRoll {
    Zero(bool),
    Delta {
        source_iter: SuccIter<MoveSource>,
        delta: Delta,
        provided_one_move: bool,
    },
}

impl PossibleMovesIter {
    pub fn new(game: GameState, roll: Roll) -> Self {
        Self {
            game,
            roll: match roll {
                Roll::Zero => PossibleMovesRoll::Zero(false),
                Roll::Delta(delta) => PossibleMovesRoll::Delta {
                    source_iter: MoveSource::succ_iter(),
                    provided_one_move: false,
                    delta,
                },
            },
        }
    }
}

impl Iterator for PossibleMovesIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.roll {
            PossibleMovesRoll::Zero(done) => {
                if *done {
                    None
                } else {
                    *done = true;
                    Some(Move::Continue {
                        game: self.game.clone(),
                        keep_turn: false,
                    })
                }
            }
            PossibleMovesRoll::Delta {
                source_iter,
                delta,
                provided_one_move,
            } => {
                let mov = source_iter.find_map(|source| self.game.move_piece(source, *delta));
                match mov {
                    Some(mov) => {
                        *provided_one_move = true;
                        Some(mov)
                    }
                    None => {
                        if *provided_one_move {
                            None
                        } else {
                            *provided_one_move = true;
                            Some(Move::Continue {
                                game: self.game.clone(),
                                keep_turn: false,
                            })
                        }
                    }
                }
            }
        }
    }
}
