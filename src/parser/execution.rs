use crate::{
    Span, StateIterator,
    lexer::{Token, TokenType},
    parser::{
        Execution, Expr, Ident, Literal, ParseError, ParseErrorType, ParseResult, Stmt, Tuple,
    },
};

pub const EXPR_TOKENS: [TokenType; 6] = [
    TokenType::IntLiteral(0),
    TokenType::FloatLiteral(0.0),
    TokenType::StringLiteral(String::new()),
    TokenType::BoolLiteral(true),
    TokenType::Tuple(Vec::new()),
    TokenType::Ident(0),
];

impl Token {
    fn to_expr(&self) -> ParseResult<Expr> {
        Ok(match self.ty.clone() {
            TokenType::IntLiteral(i) => Expr::Literal(Literal::Int(i, self.span.clone())),
            TokenType::FloatLiteral(f) => Expr::Literal(Literal::Float(f, self.span.clone())),
            TokenType::StringLiteral(f) => Expr::Literal(Literal::String(f, self.span.clone())),
            TokenType::BoolLiteral(f) => Expr::Literal(Literal::Bool(f, self.span.clone())),

            TokenType::Tuple(t) => Expr::Tuple(Tuple(todo!())),
            TokenType::Ident(id) => Expr::Ident(Ident {
                id,
                span: self.span.clone(),
            }),
            _ => unexpected_token!(EXPR_TOKENS.to_vec(), Some(self.ty.clone()), self.span),
        })
    }
}

/// MUST ENSURE NOT EMPTY
pub fn parse_execution(tokens: &[Token]) -> ParseResult<Execution> {
    Ok(match tokens {
        [] => unreachable!(),
        [t] => Execution::Single(t.to_expr()?),
        [instance, message, rest @ ..] => Execution::Called {
            instance: instance.to_expr()?,
            message: message.to_expr()?,
            args: rest
                .iter()
                .map(Token::to_expr)
                .collect::<Result<Vec<_>, _>>()?,
        },
    })
}

pub trait ParseAssignExecuteExt {
    fn parse_assign_execute(&mut self) -> ParseResult<Stmt>;
}

impl ParseAssignExecuteExt for StateIterator<'_, Token> {
    fn parse_assign_execute(&mut self) -> ParseResult<Stmt> {
        let mut eq_idx = None;
        let mut end_idx = None;
        let start_idx = self.current();

        for (i, token) in self.peek_from_current() {
            match token.ty {
                TokenType::Eq => {
                    if eq_idx.is_none() {
                        eq_idx = Some(i);
                    }
                }
                TokenType::Semicolon => {
                    end_idx = Some(i);
                    break;
                }
                _ => {}
            }
        }

        let lhs = if let Some(idx) = eq_idx {
            let l = Some(self.slice_advance(idx));
            self.advance(); // eq
            l
        } else {
            None
        };
        let rhs = if let Some(end_idx) = end_idx {
            let r = self.slice_advance(end_idx);
            self.advance(); // semicolon
            r
        } else {
            panic!("No terminal semicolon");
        };

        if let Some(lhs) = lhs {
            if lhs.is_empty() {
                return Err(ParseError {
                    ty: ParseErrorType::EmptyExpression,
                    // SAFETY: checked above
                    span: Span(start_idx..start_idx + eq_idx.unwrap()),
                });
            }
            if rhs.is_empty() {
                return Err(ParseError {
                    ty: ParseErrorType::EmptyExpression,
                    // SAFETY: checked above
                    span: Span(start_idx + eq_idx.unwrap()..start_idx + end_idx.unwrap()),
                });
            }

            Ok(Stmt::Assignment {
                lhs: parse_execution(lhs)?,
                rhs: parse_execution(rhs)?,
            })
        } else {
            if rhs.is_empty() {
                return Err(ParseError {
                    ty: ParseErrorType::EmptyExpression,
                    // SAFETY: checked above
                    span: Span(start_idx..start_idx + end_idx.unwrap()),
                });
            }

            Ok(Stmt::Exe(parse_execution(rhs)?))
        }
    }
}
