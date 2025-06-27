#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StripState(u16);

impl StripState {
    pub fn new() -> Self {
        Self(0)
    }
    pub fn set(&mut self, i: StripIndex, placed: bool) {
        *self = Self(if placed {
            self.0 | 1 << i.0
        } else {
            self.0 & !(1 << i.0)
        });
    }
    pub fn get(&self, i: StripIndex) -> bool {
        (self.0 >> i.0) & 1 != 0
    }

    pub fn count_pieces(&self) -> u8 {
        StripIndex::iter_all().filter(|&i| self.get(i)).count() as u8
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Delta(u8);

impl Delta {
    pub fn new(d: u8) -> Option<Delta> {
        if d > 4 || d < 1 {
            None
        } else {
            Some(Delta(d))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StripIndex(u8);

impl StripIndex {
    pub fn new(i: u8) -> Option<StripIndex> {
        if i > 13 {
            None
        } else {
            Some(StripIndex(i))
        }
    }
    pub fn iter_all() -> impl Iterator<Item = StripIndex> {
        (0..14).map(StripIndex)
    }

    pub fn both_teams_accessible(&self) -> bool {
        (4..=11).contains(&self.0)
    }

    pub fn apply_delta(self, d: Delta) -> DeltaResult {
        let new_i = self.0 + d.0;
        match new_i {
            i @ 0..=13 => DeltaResult::Index(StripIndex(i)),
            14 => DeltaResult::Score,
            _ => DeltaResult::OutOfBounds,
        }
    }

    fn from_delta(d: Delta) -> StripIndex {
        Self(d.0 - 1)
    }

    pub fn square(&self) -> Square {
        match self {
            StripIndex(3 | 7 | 13) => Square::Flower,
            _ => Square::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MoveSource {
    Launch,
    Index(StripIndex),
}

impl MoveSource {
    pub fn apply_delta(self, d: Delta) -> DeltaResult {
        match self {
            Self::Index(i) => i.apply_delta(d),
            Self::Launch => DeltaResult::Index(StripIndex::from_delta(d)),
        }
    }
    pub fn iter_all() -> impl Iterator<Item = Self> {
        std::iter::once(MoveSource::Launch).chain(StripIndex::iter_all().map(MoveSource::Index))
    }
}

pub enum DeltaResult {
    Index(StripIndex),
    Score,
    OutOfBounds,
}

pub enum Square {
    Flower,
    Normal,
}
