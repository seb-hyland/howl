use crate::{
    Span, StateIterator,
    lexer::{Keyword, Token, TokenType},
    parser::{execution::ParseExecutionExt, typedef::ParseTypeExt},
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
        expected: Vec<TokenType>,
        actual: Option<TokenType>,
    },
}

trait ParseExt {
    fn parse_stmt(&mut self) -> ParseResult<Option<Stmt>>;
}
impl ParseExt for StateIterator<'_, Token> {
    fn parse_stmt(&mut self) -> ParseResult<Option<Stmt>> {
        let first_token = match self.peek() {
            Some(t) => t,
            None => return Ok(None),
        };
        match first_token.ty {
            TokenType::Keyword(Keyword::Type) => {
                return self.parse_type().map(Some);
            }
            _ => self.parse_execution().map(Some),
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
                    unexpected_token!(vec![$expected], Some(token.ty.clone()), token.span);
                }
            }
            None => {
                unexpected_token!(vec![$expected], None, $last_span);
            }
        }
    }};
}

mod execution;
mod typedef;
