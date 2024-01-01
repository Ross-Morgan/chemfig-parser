pub mod error;
pub mod parser;
pub mod span;
pub mod tokens;

pub mod prelude {
    use super::{error, parser, span, tokens};

    pub use error::ChemfigParseError;
    pub use parser::ChemfigParser;
    pub use span::Span;
    pub use tokens::{ChemfigToken, ChemfigTokenStream};
}

#[macro_export]
macro_rules! parse_chemfig {
    ($src:literal) => {
        $crate::parser::ChemfigParser::parse($src)
    };
}