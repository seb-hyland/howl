use crate::{
    StateIterator,
    lexer::{Token, TokenType},
    parser::{Ident, ParseError, ParseErrorType, ParseResult, Tuple, execution::parse_execution},
};
use std::collections::HashMap;

pub fn parse_tuple(inner: &[Token]) -> ParseResult<Tuple> {
    let mut fields = HashMap::new();

    let parser = StateIterator::new(inner);
    loop {
        let ident = match parser.advance() {
            None => break,
            Some(Token {
                ty: TokenType::Ident(id),
                span,
            }) => Ident {
                id: *id,
                span: span.clone(),
            },
            Some(t) => unexpected_token!(vec![TokenType::Ident(0)], Some(t.ty.clone()), t.span),
        };
        let _eq = advance_and_assert_type!(parser, TokenType::Eq, ident.span);

        let mut comma_idx = None;
        for (i, token) in parser.peek_from_current() {
            if matches!(token.ty, TokenType::Comma) {
                comma_idx = Some(i);
                break;
            }
        }

        let tokens = if let Some(idx) = comma_idx {
            let s = parser.slice_advance(idx);
            let _comma = parser.advance();
            s
        } else {
            parser.slice_advance_to_end()
        };
        if tokens.is_empty() {
            return Err(ParseError {
                ty: ParseErrorType::EmptyExpression,
                span: todo!(),
            });
        }
        let exe = parse_execution(tokens)?;
        fields.insert(ident, exe);
    }

    Ok(Tuple(fields))
}
