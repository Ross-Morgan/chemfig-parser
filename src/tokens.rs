use std::ops::RangeBounds;
use std::slice::Iter;
use std::vec::Drain;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChemfigToken {
    Element(String),
    Bond(usize),
    Group(Vec<ChemfigToken>),
    Ring(Vec<ChemfigToken>, usize),
}

#[derive(Clone, Debug)]
pub struct ChemfigTokenStream(Vec<ChemfigToken>);

impl ChemfigTokenStream {
    pub const fn new() -> Self {
        Self(vec![])
    }

    pub fn token_count(&self) -> usize {
        self.0.len()
    }

    pub fn push_token(&mut self, token: ChemfigToken) {
        self.0.push(token);
    }

    pub fn remove(&mut self, idx: usize) -> ChemfigToken {
        self.0.remove(idx)
    }

    pub fn remove_range<R: RangeBounds<usize>>(&mut self, r: R) -> Drain<'_, ChemfigToken> {
        self.0.drain(r)
    }

    pub fn iter(&self) -> Iter<'_, ChemfigToken> {
        self.0.iter()
    }
}
