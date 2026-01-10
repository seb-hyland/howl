use std::ops::Range;

pub mod lexer;
pub mod parser;

#[derive(Debug)]
pub struct Span(Range<usize>);
