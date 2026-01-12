use crate::{
    Span,
    lexer::{Keyword, Token, TokenType},
};

pub enum Stmt {
    TypeDefinition {
        name: Ident,
        instance_fields: Vec<(Ident, Option<RestrictionStmt>)>,
        type_fields: Vec<(Ident, Option<RestrictionStmt>)>,
    },
    TraitDefinition {
        name: Ident,
        handlers: Vec<Ident>,
    },
    Assignment {
        lhs: (Execution, Option<RestrictionStmt>),
        rhs: Execution,
    },
    Return(Execution),
    Exe(Execution),
}

pub struct RestrictionStmt {
    pub stmt: Box<Expr>,
    pub span: Span,
}

pub struct Execution(pub Vec<Expr>);

pub enum Expr {
    Ident(Ident),
    Tuple(Tuple),
    Literal(Literal),
    Lambda(Lambda),
    Block(Block),
}

pub enum Tuple {
    And(Vec<Expr>),
    Map(Vec<(Ident, Expr)>),
    Or(Vec<Expr>),
}

pub struct Ident {
    pub id: usize,
    pub span: Span,
}
pub enum Literal {
    IntLiteral(i64, Span),
    FloatLiteral(f64, Span),
    StringLiteral(String, Span),
}
pub struct Lambda {
    pub args: Vec<(Ident, Option<RestrictionStmt>)>,
    pub return_restrict: Option<RestrictionStmt>,
    pub body: Vec<Stmt>,
    pub span: Span,
}
pub struct Block {
    pub body: Vec<Stmt>,
    pub span: Span,
}

type ParseResult<T> = Result<T, ParseError>;

pub struct ParseError {
    ty: ParseErrorType,
    span: Span,
}

enum ParseErrorType {}

pub struct Parser {
    buf: Vec<Token>,
    current: usize,
    expressions: Vec<Expr>,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.buf.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        let cur_byte = self.buf.get(self.current);
        if cur_byte.is_some() {
            self.current += 1;
        }
        cur_byte
    }

    fn parse_stmt(&mut self) -> ParseResult<Option<Stmt>> {
        let first_token = match self.peek() {
            Some(t) => t,
            None => return Ok(None),
        };
        match first_token.ty {
            TokenType::Keyword(k) if k == Keyword::Type => {
                return self.parse_type().map(|v| Some(v));
            }
            TokenType::Keyword(k) if k == Keyword::Trait => {
                return self.parse_trait().map(|v| Some(v));
            }
            _ => {}
        };

        todo!("")
    }

    fn parse_type(&mut self) -> ParseResult<Stmt> {}

    fn parse_trait(&mut self) -> ParseResult<Stmt> {}
}
