use thiserror::Error;

use crate::span::Span;

#[derive(Error, Debug)]
pub enum ChemfigParseError {
    #[error("Invalid char {0}")]
    InvalidChar(char),
    #[error("Unenclosed group, expected {expected:?}, found {found:?} at {}", .span.from)]
    UnenclosedGroup {
        expected: char,
        found: char,
        span: Span,
    },
    #[error("Expected {0}, found `EOF`")]
    UnexpectedEOF(char),
    #[error("Expected bond identifier (-, =, ≡, -[n]), found {0}")]
    InvalidBondIdent(char),
}
