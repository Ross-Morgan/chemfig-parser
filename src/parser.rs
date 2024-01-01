use std::str::CharIndices;
use std::iter::Peekable;

use crate::error::ChemfigParseError;
use crate::span::Span;
use crate::tokens::{ChemfigTokenStream, ChemfigToken};

pub struct ChemfigParser<'a> {
    char_stream: Peekable<CharIndices<'a>>,
    tokens: ChemfigTokenStream,
}

impl<'a> ChemfigParser<'a> {
    pub fn parse(src: &'a str) -> Result<ChemfigTokenStream, ChemfigParseError> {
        let mut parser = Self::new(src);

        parser.parse_stream()?;

        Ok(parser.finalise())
    }

    fn new(src: &'a str) -> Self {
        Self { char_stream: src.char_indices().peekable(), tokens: ChemfigTokenStream::new(), }
    }
}

impl<'a> ChemfigParser<'a> {
    fn parse_stream(&mut self) -> Result<(), ChemfigParseError> {
        while let Some(&(idx, c)) = self.char_stream.peek() {
            self.parse_one(c, idx)?;
        }

        Ok(())
    }

    fn parse_one(&mut self, c: char, idx: usize) -> Result<(), ChemfigParseError> {
        match c {
            '*' => {
                let _ = self.char_stream.next();
                self.parse_group(true)?;
            },
            '(' => self.parse_group(false)?,
            c if c.is_ascii_uppercase() => self.parse_element()?,
            '-' | '=' | '≡' => self.parse_bond()?,
            '_' => self.parse_number()?,
            _ => Err(ChemfigParseError::InvalidChar { c, span: Span::new(idx, idx + 1) })?,
        };

        Ok(())
    }

    fn parse_group(&mut self, is_ring: bool) -> Result<(), ChemfigParseError> {
        match self.char_stream.next() {
            Some((_, '(')) => Ok(()),
            Some((idx, c)) => Err(ChemfigParseError::UnenclosedGroup { expected: '(', found: c, span: Span::new(idx, idx + 1) }),
            None => Err(ChemfigParseError::UnexpectedEOF)?,
        }?;

        let token_len = self.tokens.token_count();

        while let Some(&(idx, c)) = self.char_stream.peek() {
            match c {
                ')' => break,
                _ => self.parse_one(c, idx)?,
            };
        }

        let (last_idx, _) = self.char_stream.next().unwrap();

        let new_len = self.tokens.token_count();

        let group_tokens = self.tokens.remove_range(token_len..new_len).collect::<Vec<_>>();

        if !is_ring {
            self.tokens.push_token(ChemfigToken::Group(group_tokens));
        } else {
            let bond_token = self.tokens.remove(self.tokens.iter().count() - 1);

            let bond_order = match bond_token {
                ChemfigToken::Bond(bo) => bo,
                _ => {
                    Err(ChemfigParseError::RingMissingBond(Span::new(last_idx, last_idx + 1)))?
                },
            };

            self.tokens.push_token(ChemfigToken::Ring(group_tokens, bond_order));
        }

        Ok(())
    }

    fn parse_element(&mut self) -> Result<(), ChemfigParseError> {
        let mut buf = String::new();

        match self.char_stream.next() {
            Some((_, c)) if c.is_ascii_uppercase() => buf.push(c),
            Some((idx, c)) => Err(ChemfigParseError::InvalidElementIdent(c, Span::new(idx, idx + 1)))?,
            None => Err(ChemfigParseError::UnexpectedEOF)?,
        };

        while let Some(&(_, c)) = self.char_stream.peek() {
            if !c.is_ascii_lowercase() {
                break;
            }

            buf.push(c);

            let _ = self.char_stream.next().expect("We literally just peeked this :skull:");
        }

        self.tokens.push_token(ChemfigToken::Element(buf));

        Ok(())
    }

    fn parse_number(&mut self) -> Result<(), ChemfigParseError> {
        let start_idx = match self.char_stream.next() {
            Some((idx, '_')) => idx,
            Some((idx, c)) => Err(ChemfigParseError::InvalidChar { c, span: Span::new(idx, idx + 1) })?,
            None => Err(ChemfigParseError::UnexpectedEOF)?,
        };

        let mut n = 0usize;

        while let Some(&(_, c @ '0'..='9')) = self.char_stream.peek() {
            let d = c.to_digit(10).expect("Bounds guaranteed") as usize;

            n *= 10;
            n += d;

            let _ = self.char_stream.next().expect("Just peeked it");
        }

        let to_be_repeated = match self.tokens.iter().last() {
            Some(t) => t.clone(),
            None => Err(ChemfigParseError::NothingToRepeat(Span::new(start_idx, start_idx + 1)))?,
        };

        if n == 0 {
            return Err(ChemfigParseError::ZeroRepetitionCount(Span::new(start_idx, start_idx + 1)));
        }

        for _ in 0..(n - 1) {
            self.tokens.push_token(to_be_repeated.clone());
        }

        Ok(())
    }

    fn parse_bond(&mut self) -> Result<(), ChemfigParseError> {
        match self.char_stream.next() {
            Some((_, '=')) => { self.tokens.push_token(ChemfigToken::Bond(2)); return Ok(()) },
            Some((_, '≡')) => { self.tokens.push_token(ChemfigToken::Bond(3)); return Ok(()) },
            Some((_, '-')) => (),
            Some((idx, c)) => Err(ChemfigParseError::InvalidBondIdent(c, Span::new(idx, idx + 1)))?,
            None => Err(ChemfigParseError::UnexpectedEOF)?,
        };

        if let Some(&(_, c)) = self.char_stream.peek() {
            match c {
                '[' => (),
                _ => {
                    self.tokens.push_token(ChemfigToken::Bond(1));
                    return Ok(());
                } ,
            };
        }

        let _ = self.char_stream.next().expect("We know [ is the next char");

        let c = self.char_stream.clone().take_while(|&(_, c)| c.is_digit(10)).collect::<Vec<_>>();

        for _ in 0..(c.len()) {
            let _ = self.char_stream.next();
        }

        match self.char_stream.peek() {
            Some(&(_, ']')) => self.char_stream.next(),
            Some(&(idx, c)) => Err(ChemfigParseError::UnenclosedGroup { expected: ']', found: c, span: Span::new(idx, idx + 1) })?,
            None => Err(ChemfigParseError::UnexpectedEOF)?,
        };

        let bond_size = c.into_iter().fold(0usize, |acc, (_, c)| {
            acc * 10 + (c.to_digit(10).expect("We know it is a digit from `take_while`") as usize)
        });

        self.tokens.push_token(ChemfigToken::Bond(bond_size));

        Ok(())
    }

    /// Returns TokenTree containing all tokens parsed from source
    pub fn finalise(self) -> ChemfigTokenStream {
        self.tokens
    }

    /// Returns TokenTree like `finalise` along with the char stream since it is assumed it has not been fully exhausted
    pub fn early_finalise(self) -> (ChemfigTokenStream, Peekable<CharIndices<'a>>) {
        (self.tokens, self.char_stream)
    }
}
