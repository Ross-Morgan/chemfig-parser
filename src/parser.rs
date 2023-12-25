use std::str::CharIndices;
use std::iter::Peekable;

use crate::error::ChemfigParseError;
use crate::span::Span;

pub struct ChemfigParser<'a> {
    char_stream: Peekable<CharIndices<'a>>,
    tokens: ChemfigTokenTree,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChemfigToken {
    Element(&'static str),
    AtomCount(usize),
    Bond(usize),
    Group(Vec<ChemfigToken>),
    GroupOpen,
    GroupClose,
}

impl<'a> ChemfigParser<'a> {
    pub fn parse(src: &'a str) -> Result<ChemfigTokenTree, ChemfigParseError> {
        let mut parser = Self::new(src);

        parser.parse_stream()?;

        Ok(parser.finalise())
    }

    fn new(src: &'a str) -> Self {
        Self { char_stream: src.char_indices().peekable(), tokens: ChemfigTokenTree(vec![]), }
    }
}

impl<'a> ChemfigParser<'a> {
    fn parse_stream(&mut self) -> Result<(), ChemfigParseError> {
        while let Some(&(_, c)) = self.char_stream.peek() {
            self.parse_one(c)?;
        }

        Ok(())
    }

    fn parse_one(&mut self, c: char) -> Result<(), ChemfigParseError> {
        match c {
            '(' => self.parse_group()?,
            c if c.is_ascii_alphabetic() => self.parse_element()?,
            '-' | '=' | '≡' => self.parse_bond()?, 
            '_' => self.parse_number()?,
            _ => Err(ChemfigParseError::InvalidChar(c))?,
        };

        Ok(())
    }

    fn parse_group(&mut self) -> Result<(), ChemfigParseError> {
        match self.char_stream.next() {
            Some((_, '(')) => Ok(()),
            Some((idx, c)) => Err(ChemfigParseError::UnenclosedGroup { expected: '(', found: c, span: Span::new(idx, idx + 1) }),
            None => Err(ChemfigParseError::UnexpectedEOF('(')),
        }?;

        self.tokens.push_token(ChemfigToken::GroupOpen);

        'peek: while let Some(&(_, c)) = self.char_stream.peek() {
            match c {
                ')' => break 'peek,
                _ => self.parse_one(c)?,
            };
        }

        self.tokens.push_token(ChemfigToken::GroupClose);

        Ok(())
    }

    fn parse_element(&mut self) -> Result<(), ChemfigParseError> {
        Ok(())
    }

    fn parse_number(&mut self) -> Result<(), ChemfigParseError> {
        Ok(())
    }

    fn parse_bond(&mut self) -> Result<(), ChemfigParseError> {
        match self.char_stream.next() {
            Some((_, '=')) => self.tokens.push_token(ChemfigToken::Bond(2)),
            Some((_, '≡')) => self.tokens.push_token(ChemfigToken::Bond(3)),
            Some((_, '-')) => (),
            Some((_, c)) => Err(ChemfigParseError::InvalidBondIdent(c))?,
            None => Err(ChemfigParseError::UnexpectedEOF('-'))?,
        };

        if let Some(&(_, c)) = self.char_stream.peek() {
            match c {
                '[' => (),
                _ => self.tokens.push_token(ChemfigToken::Bond(1)),
            };
        }

        let _ = self.char_stream.next().expect("We know [ is the next char");

        let c = self.char_stream.clone().take_while(|&(_, c)| c.is_digit(10) || c == ']').collect::<Vec<_>>();

        let bond_size = c.into_iter().fold(0usize, |acc, (_, c)| {
            acc * 10 + (c.to_digit(10).expect("We know it is a digit from `take_while`") as usize)
        });

        self.tokens.push_token(ChemfigToken::Bond(bond_size));

        Ok(())
    }

    /// Returns TokenTree containing all tokens parsed from source
    pub fn finalise(self) -> ChemfigTokenTree {
        self.tokens
    }

    /// Returns TokenTree like `finalise` along with the char stream since it is assumed it has not been fully exhausted
    pub fn early_finalise(self) -> (ChemfigTokenTree, Peekable<CharIndices<'a>>) {
        (self.tokens, self.char_stream)
    }
}



pub struct ChemfigTokenTree(Vec<ChemfigToken>);

impl ChemfigTokenTree {
    pub const fn new() -> Self {
        Self(vec![])
    }

    pub fn push_token(&mut self, token: ChemfigToken) {
        self.0.push(token);
    }
}

fn f() {
    let tree = ChemfigParser::parse("C(-H)");

}