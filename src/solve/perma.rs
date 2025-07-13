use std::cmp::{self, Ordering};

use crate::{
    game::{
        strip::{StripIndex, StripState},
        GameState, GameStateSmall, TeamState,
    },
    successor::Succ,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct PermaKey {
    pub team_gt: PermaTeamKey,
    pub team_lt: PermaTeamKey,
    max_token: MaxToken,
}

impl PermaKey {
    pub fn new(game: GameState) -> Self {
        let prot = (PermaTeamKey::new(game.prot), game.prot.strip.start_bits());
        let opp = (PermaTeamKey::new(game.opp), game.opp.strip.start_bits());
        let ((team_gt, strip_start_gt), (team_lt, strip_start_lt)) =
            if prot > opp { (prot, opp) } else { (opp, prot) };

        let max_token = MaxToken::new(
            game.prot.strip,
            game.opp.strip,
            strip_start_gt,
            strip_start_lt,
        );
        PermaKey {
            team_gt,
            team_lt,
            max_token,
        }
    }

    pub fn reachable_in_one_move_from(&self, other: PermaKey) -> bool {
        match (
            self.team_gt.score.checked_sub(other.team_gt.score),
            self.team_lt.score.checked_sub(other.team_lt.score),
        ) {
            (Some(0), Some(0)) | (Some(1), Some(0)) | (Some(0), Some(1)) => true,
            _ => false,
        }
    }
}

impl From<GameState> for PermaKey {
    fn from(value: GameState) -> Self {
        Self::new(value)
    }
}

impl From<&GameState> for PermaKey {
    fn from(value: &GameState) -> Self {
        Self::new(*value)
    }
}

impl From<GameStateSmall> for PermaKey {
    fn from(value: GameStateSmall) -> Self {
        Self::new(GameState::from(value))
    }
}
impl From<&GameStateSmall> for PermaKey {
    fn from(value: &GameStateSmall) -> Self {
        Self::new(GameState::from(*value))
    }
}

impl PartialOrd for PermaKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PermaKey {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (
            self.team_gt.score,
            self.team_gt.strip_end,
            self.team_lt.score,
            self.team_lt.strip_end,
            self.max_token,
        )
            .cmp(&(
                other.team_gt.score,
                other.team_gt.strip_end,
                other.team_lt.score,
                other.team_lt.strip_end,
                other.max_token,
            ))
            .reverse()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct MaxToken {
    focus_token: FocusToken,
    max_is_flower: bool,
}

impl MaxToken {
    pub fn new(
        prot_strip: StripState,
        opp_strip: StripState,
        strip_start_gt: u8,
        strip_start_lt: u8,
    ) -> Self {
        let index = StripIndex::succ_iter()
            .take(12)
            .filter(|i| prot_strip.get(*i) || opp_strip.get(*i))
            .last();
        match index {
            None | Some(StripIndex(..4)) => MaxToken {
                focus_token: FocusToken::Start {
                    strip_start_gt,
                    strip_start_lt,
                },
                max_is_flower: false,
            },
            Some(StripIndex(7)) => {
                let second_index = StripIndex::succ_iter()
                    .take(7)
                    .filter(|i| prot_strip.get(*i) || opp_strip.get(*i))
                    .last();
                MaxToken {
                    focus_token: match second_index {
                        None | Some(StripIndex(..4)) => FocusToken::Start {
                            strip_start_gt,
                            strip_start_lt,
                        },
                        Some(i) => FocusToken::Shared(i),
                    },
                    max_is_flower: true,
                }
            }
            Some(i) => MaxToken {
                focus_token: FocusToken::Shared(i),
                max_is_flower: false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum FocusToken {
    Start {
        strip_start_gt: u8,
        strip_start_lt: u8,
    },
    Shared(StripIndex),
}
impl PartialOrd for FocusToken {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FocusToken {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Self::Start {
                    strip_start_gt,
                    strip_start_lt,
                },
                Self::Start {
                    strip_start_gt: other_gt,
                    strip_start_lt: other_lt,
                },
            ) => (strip_start_gt, strip_start_lt).cmp(&(other_gt, other_lt)),
            (Self::Start { .. }, Self::Shared(_)) => Ordering::Less,
            (Self::Shared(_), Self::Start { .. }) => Ordering::Greater,
            (Self::Shared(i), Self::Shared(j)) => i.cmp(j),
        }
    }
}

impl Ord for MaxToken {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.max_is_flower, other.max_is_flower) {
            (true, true) => self.focus_token.cmp(&other.focus_token),
            (false, false) => self.focus_token.cmp(&other.focus_token),
            (true, false) => FocusToken::Shared(StripIndex(7)).cmp(&other.focus_token),
            (false, true) => self.focus_token.cmp(&FocusToken::Shared(StripIndex(7))),
        }
    }
}

impl PartialOrd for MaxToken {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct PermaTeamKey {
    pub score: u8,
    strip_end: u8,
}

impl PermaTeamKey {
    pub fn new(team: TeamState) -> Self {
        let strip_end = team.strip.end_bits();
        Self {
            score: team.score,
            strip_end,
        }
    }
}
