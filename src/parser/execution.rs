use crate::{
    StateIterator,
    lexer::{Token, TokenType},
    parser::{Execution, ParseResult, Stmt},
};

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

pub trait ParseExecutionExt {
    fn parse_execution(&mut self) -> ParseResult<Stmt>;
}

impl ParseExecutionExt for StateIterator<'_, Token> {
    fn parse_execution(&mut self) -> ParseResult<Stmt> {
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
            Ok(Some(Stmt::Assignment { lhs, rhs }))
        } else {
            Ok(Some(Stmt::Exe(rhs)))
        }
    }
}
