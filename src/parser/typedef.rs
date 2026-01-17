use crate::{
    lexer::{Keyword, TokenType},
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

        let _open = advance_and_assert_type!(self, TokenType::OpenParen, name.span);

        let mut last_span = &_open.span;
        let mut instance_fields = Vec::new();
        let mut type_fields = Vec::new();

        loop {
            match self.advance() {
                None => unexpected_token!(
                    &[
                        TokenType::Ident(0),
                        TokenType::TypeIdent(0),
                        TokenType::Keyword(Keyword::Opaque),
                        TokenType::CloseParen
                    ],
                    None,
                    last_span
                ),
                Some(token) => match token.ty {
                    TokenType::CloseParen => {
                        break;
                    }
                    TokenType::Ident(id) | TokenType::TypeIdent(id) => {
                        let mut push_vec = match token.ty {
                            TokenType::Ident(_) => &mut instance_fields,
                            TokenType::TypeIdent(_) => &mut type_fields,
                            _ => unreachable!(),
                        };
                        let field_ident = Ident {
                            id,
                            span: token.span.clone(),
                        };
                        match self.advance() {
                            Some(next_token) => match next_token.ty {
                                TokenType::Colon => Some(self.parse_restriction()?),
                                TokenType::Comma => None,
                            },
                        };
                    }
                    _ => {}
                },
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
