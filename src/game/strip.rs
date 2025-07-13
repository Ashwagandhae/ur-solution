use crate::successor::Succ;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StripState(pub u16);

impl Default for StripState {
    fn default() -> Self {
        Self::new()
    }
}

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
        StripIndex::succ_iter().filter(|&i| self.get(i)).count() as u8
    }

    pub fn from_start_and_end(start: u8, end: u8) -> Self {
        let start_bits = (start & 0b1111) as u16; // bits 0..=3
        let end_bits = ((end & 0b11) as u16) << 12; // bits 12..=13
        Self(start_bits | end_bits)
    }

    pub fn start_bits(&self) -> u8 {
        ((self.0) & 0b0000_0000_0000_1111) as u8
    }

    pub fn end_bits(&self) -> u8 {
        ((self.0 >> 12) & 0b11) as u8
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Delta(u8);

impl Delta {
    pub fn new(d: u8) -> Option<Delta> {
        if !(1..=4).contains(&d) {
            None
        } else {
            Some(Delta(d))
        }
    }

    pub fn get(&self) -> u8 {
        self.0
    }
}

impl Succ for Delta {
    fn first() -> Self {
        Self(1)
    }

    fn succ(&self) -> Option<Self> {
        Self::new(self.0 + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StripIndex(pub u8);

impl StripIndex {
    pub fn new(i: u8) -> Option<StripIndex> {
        if i > 13 {
            None
        } else {
            Some(StripIndex(i))
        }
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

impl Succ for StripIndex {
    fn first() -> Self {
        Self(0)
    }
    fn succ(&self) -> Option<Self> {
        Self::new(self.0 + 1)
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
}

impl Succ for MoveSource {
    fn first() -> Self {
        MoveSource::Launch
    }

    fn succ(&self) -> Option<Self> {
        match self {
            Self::Launch => Some(Self::Index(StripIndex::first())),
            Self::Index(index) => Some(Self::Index(index.succ()?)),
        }
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
