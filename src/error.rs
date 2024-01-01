use thiserror::Error;

use crate::span::Span;

#[derive(Error, Debug)]
pub enum ChemfigParseError {
    #[error("Invalid char {} at {}", .c, .span.from)]
    InvalidChar {
        c: char,
        span: Span,
    },
    #[error("Unenclosed group, expected {expected:?}, found {found:?} at {}", .span.from)]
    UnenclosedGroup {
        expected: char,
        found: char,
        span: Span,
    },
    #[error("Expected char, found `EOF`")]
    UnexpectedEOF,
    #[error("Expected bond identifier (-, =, ≡, -[n]), found {0}")]
    InvalidBondIdent(char, Span),
    #[error("Expected element ident, found {0}")]
    InvalidElementIdent(char, Span),
    #[error("Expected an item to repeat at or before {}", .0.from)]
    NothingToRepeat(Span),
    #[error("Tried to repeat item zero times at {}", .0.from)]
    ZeroRepetitionCount(Span),
    #[error("Ring missing final bond at {}", .0.from)]
    RingMissingBond(Span),
}
