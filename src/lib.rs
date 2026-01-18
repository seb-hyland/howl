use std::ops::Range;

pub mod lexer;
pub mod parser;

#[derive(PartialEq, Debug, Clone)]
pub struct Span(Range<usize>);
