use std::{cell::UnsafeCell, slice};

use crate::{
    Span,
    lexer::{Keyword, Token, TokenType},
};

pub enum Stmt {
    TypeDefinition {
        name: Ident,
        instance_fields: Vec<Ident>,
        type_fields: Vec<Ident>,
    },
    // TraitDefinition {
    //     name: Ident,
    //     handlers: Vec<Ident>,
    // },
    Assignment {
        lhs: Execution,
        rhs: Execution,
    },
    Exe(Execution),
}

pub enum Execution {
    Single(Expr),
    Called {
        instance: Expr,
        message: Expr,
        args: Vec<Expr>,
    },
}

pub enum Expr {
    Ident(Ident),
    Literal(Literal),
    Tuple(Tuple),
}

pub struct Tuple(Vec<Expr>);

pub struct Ident {
    pub id: usize,
    pub span: Span,
}
pub enum Literal {
    IntLiteral(i64, Span),
    FloatLiteral(f64, Span),
    StringLiteral(String, Span),
    BoolLiteral(bool, Span),
}

type ParseResult<T> = Result<T, ParseError>;

pub struct ParseError {
    ty: ParseErrorType,
    span: Span,
}

enum ParseErrorType {
    UnexpectedToken {
        expected: &'static [TokenType],
        actual: Option<TokenType>,
    },
}

pub struct Parser {
    buf: Vec<Token>,
    current: UnsafeCell<usize>,
    expressions: Vec<Expr>,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.buf.get(unsafe { *self.current.get() })
    }

    fn advance(&self) -> Option<&Token> {
        let ptr = self.current.get();

        let cur_byte = self.buf.get(unsafe { *ptr });
        if cur_byte.is_some() {
            unsafe { *ptr += 1 };
        }
        cur_byte
    }

    fn peek_from_current(&self) -> impl IntoIterator<Item = (usize, &Token)> {
        self.buf[unsafe { *self.current.get() }..]
            .iter()
            .enumerate()
    }

    fn slice_from_current(&self, end: usize) -> &[Token] {
        if end == 0 {
            todo!()
        }
        unsafe {
            *self.current.get() += end - 1;
        }
        &self.buf[unsafe { *self.current.get() }..end]
    }

    fn parse_execution(tokens: &[Token]) -> Execution {
        match tokens {
            [t] => Execution::Single(t),
            [instance, message, rest @ ..] => Execution::Called {
                instance,
                message,
                args: rest,
            },
        }
    }

    fn parse_stmt(&mut self) -> ParseResult<Option<Stmt>> {
        let first_token = match self.peek() {
            Some(t) => t,
            None => return Ok(None),
        };
        match first_token.ty {
            TokenType::Keyword(Keyword::Type) => {
                return self.parse_type().map(Some);
            }
            _ => {
                let mut eq_idx = None;
                let mut end_idx = None;

                for (i, token) in self.peek_from_current() {
                    match token.ty {
                        TokenType::Eq => {
                            eq_idx = Some(i);
                        }
                        TokenType::Semicolon => {
                            end_idx = Some(i);
                            break;
                        }
                        _ => {}
                    }
                }

                let lhs = if let Some(idx) = eq_idx {
                    let l = Some(self.slice_from_current(idx));
                    self.advance(); // eq
                    l
                } else {
                    None
                };
                let rhs = if let Some(end_idx) = end_idx {
                    let r = self.slice_from_current(end_idx);
                    self.advance(); // semicolon
                    r
                } else {
                    panic!("No terminal semicolon");
                };

                if let Some(lhs) = lhs {
                    Ok(Some(Stmt::Assignment { lhs, rhs }))
                } else {
                    Ok(Some(Stmt::Exe(rhs)))
                }
            }
        }
    }
}

macro_rules! unexpected_token {
    ($expected:expr, $actual:expr, $span:expr) => {
        return Err($crate::parser::ParseError {
            ty: $crate::parser::ParseErrorType::UnexpectedToken {
                expected: $expected,
                actual: $actual,
            },
            span: $span.clone(),
        })
    };
}

macro_rules! advance_and_assert_type {
    ($self:expr, $expected:expr, $last_span:expr) => {{
        let next = $self.advance();
        match next {
            Some(token) => {
                if std::mem::discriminant(&token.ty) == std::mem::discriminant(&$expected) {
                    token
                } else {
                    unexpected_token!(&[$expected], Some(token.ty.clone()), token.span);
                }
            }
            None => {
                unexpected_token!(&[$expected], None, $last_span);
            }
        }
    }};
}

mod typedef;
