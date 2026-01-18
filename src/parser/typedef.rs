use crate::{
    StateIterator,
    lexer::{Keyword, Token, TokenType},
    parser::{Ident, ParseResult, Stmt},
};

pub trait ParseTypeExt {
    fn parse_type(&mut self) -> ParseResult<Stmt>;
}

impl ParseTypeExt for StateIterator<'_, Token> {
    fn parse_type(&mut self) -> ParseResult<Stmt> {
        let _type_ident = self.advance().unwrap();
        assert!(_type_ident.ty == TokenType::Keyword(Keyword::Type));

        let name = advance_and_assert_type!(self, TokenType::Ident(0), _type_ident.span);
        let name = Ident {
            id: match name.ty {
                TokenType::Ident(id) => id,
                _ => unreachable!(),
            },
            span: name.span.clone(),
        };

        let (tuple, mut last_span) = match self.advance() {
            Some(Token {
                ty: TokenType::Tuple(p),
                span,
            }) => (p, span),
            other => unexpected_token!(
                vec![TokenType::Tuple(Vec::new())],
                other.map(|o| o.ty.clone()),
                name.span
            ),
        };
        let mut instance_fields = Vec::new();
        let mut type_fields = Vec::new();
        let tuple_parser = StateIterator::new(&tuple);

        loop {
            let field_token = match tuple_parser.advance() {
                None => unexpected_token!(
                    vec![TokenType::Ident(0), TokenType::TypeIdent(0)],
                    None,
                    last_span
                ),
                Some(token) => token,
            };
            last_span = &field_token.span;
            match field_token.ty {
                TokenType::Ident(id) | TokenType::TypeIdent(id) => {
                    let push_vec = match field_token.ty {
                        TokenType::Ident(_) => &mut instance_fields,
                        TokenType::TypeIdent(_) => &mut type_fields,
                        _ => unreachable!(),
                    };
                    let field_ident = Ident {
                        id,
                        span: field_token.span.clone(),
                    };
                    push_vec.push(field_ident);
                }
                _ => {
                    unexpected_token!(
                        vec![TokenType::Ident(0), TokenType::TypeIdent(0),],
                        Some(field_token.ty.clone()),
                        field_token.span
                    )
                }
            };

            match tuple_parser.advance() {
                Some(Token {
                    ty: TokenType::Comma,
                    ..
                }) => {}
                None => break,
                other => unexpected_token!(
                    vec![TokenType::Comma],
                    other.map(|o| o.ty.clone()),
                    field_token.span
                ),
            }
        }
        let _stmt_end = advance_and_assert_type!(self, TokenType::Semicolon, last_span);

        Ok(Stmt::TypeDefinition {
            name,
            instance_fields,
            type_fields,
        })
    }
}
