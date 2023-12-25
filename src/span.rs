#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub(crate) from: usize,
    pub(crate) to: usize,
}

impl Span {
    pub const fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    } 
}
