use crate::{
    lexer::{Keyword, Token, TokenType},
    parser::{Ident, ParseResult, Parser, Stmt},
};

impl Parser {
    pub fn parse_type(&mut self) -> ParseResult<Stmt> {
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
                ty: TokenType::Parenthesised(p),
                span,
            }) => (p, span),
            other => unexpected_token!(
                &[TokenType::Parenthesised(Vec::new())],
                other.map(|o| o.ty),
                name.span
            ),
        };
        let mut instance_fields = Vec::new();
        let mut type_fields = Vec::new();

        loop {
            let field_token = match self.advance() {
                None => unexpected_token!(
                    &[TokenType::Ident(0), TokenType::TypeIdent(0)],
                    None,
                    last_span
                ),
                Some(token) => token,
            };
            match field_token.ty {
                TokenType::CloseParen => {
                    break;
                }
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

                    let _comma_or_closeparen = match self.advance() {
                        Some(t) => t,
                        None => unexpected_token!(
                            &[TokenType::Comma, TokenType::CloseParen],
                            None,
                            last_span
                        ),
                    };
                    match _comma_or_closeparen.ty {
                        TokenType::Comma => last_span = &_comma_or_closeparen.span,
                        TokenType::CloseParen => break,
                        _ => unexpected_token!(
                            &[TokenType::Comma, TokenType::CloseParen],
                            Some(_comma_or_closeparen.ty.clone()),
                            _comma_or_closeparen.span
                        ),
                    };
                }
                _ => {
                    unexpected_token!(
                        &[TokenType::Ident(0), TokenType::TypeIdent(0),],
                        Some(field_token.ty.clone()),
                        field_token.span
                    )
                }
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
