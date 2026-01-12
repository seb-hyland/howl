use crate::{Span, lexer::Token};

pub enum Stmt<'src> {
    TypeDefinition {
        name: Ident<'src>,
        instance_fields: Vec<(Ident<'src>, Option<RestrictionStmt<'src>>)>,
        type_fields: Vec<(Ident<'src>, Option<RestrictionStmt<'src>>)>,
    },
    TraitDefinition {
        name: Ident<'src>,
        handlers: Vec<Ident<'src>>,
    },
    Assignment {
        lhs: (Ident<'src>, Option<RestrictionStmt<'src>>),
        rhs: Vec<Expr<'src>>,
    },
    Execution(Vec<Expr<'src>>),
}

pub struct RestrictionStmt<'src> {
    stmt: Box<Expr<'src>>,
    span: Span,
}

pub enum Expr<'src> {
    Ident(Ident<'src>),
    Tuple(Vec<Expr<'src>>),
    Literal(Literal<'src>),
    Lambda(Lambda<'src>),
    Block(Block<'src>),
}

pub enum Tuple<'src> {
    And(Vec<Expr<'src>>),
}

pub struct Ident<'src> {
    name: &'src [u8],
    span: Span,
}
pub enum Literal<'src> {
    IntLiteral(i64, Span),
    FloatLiteral(f64, Span),
    StringLiteral(&'src [u8], Span),
}
pub struct Lambda<'src> {
    args: Vec<(Ident<'src>, Option<RestrictionStmt<'src>>)>,
    return_restrict: Option<RestrictionStmt<'src>>,
    body: Vec<Stmt<'src>>,
    span: Span,
}
pub struct Block<'src> {
    body: Vec<Stmt<'src>>,
    span: Span,
}

pub struct Parser<'src> {
    buf: Token,
    current: usize,
    expressions: Vec<Expr<'src>>,
}
