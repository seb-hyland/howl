use crate::IdentArena;

pub enum Keyword {
    Type,
}

#[derive(Clone, Copy, Debug)]
pub struct Ident {
    pub id: u64,
}

#[derive(Debug)]
pub enum Stmt {
    Assignment { dst: Ident, rhs: Execution },
    Exe(Execution),
}

#[derive(Debug)]
pub enum Execution {
    Single(Expr),
    Called(Expr, Expr, Vec<Expr>),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Ident(Ident),
    Lit(Literal),
}

#[derive(Clone, Debug)]
pub enum Literal {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
}

peg::parser! {
    pub grammar howl_parser(arena: &mut IdentArena) for str {
        // Helpers
        rule wsp() = quiet! { [c if c.is_ascii_whitespace()]+ {} }
        rule _() = quiet! { wsp()* }
        rule digit() -> &'input str = quiet! { $[c if c.is_ascii_digit()] } / expected!("digit")
        rule terminator() = ['(' | ')' | '[' | ']' | ',' | ':' | ';' | '|' | '"'] / wsp()
            rule eq() = quiet!{ "=" } / expected!("EQUAL")

        // Atoms
        rule int_literal() -> i32 = n:$(digit()+) { ? n.parse().or(Err("i32")) }
        rule float_literal() -> f64 = n:$(digit()+ "." digit()*) { ? n.parse().or(Err("f64")) }
        rule bool_literal() -> bool = "true" { true } / "false" { false }
        rule str_literal() -> &'input str = "\"" s:$((!"\"" [_])*) "\"" { s }
        rule keyword() -> Keyword = "type" &terminator() { Keyword::Type }
        rule identifier() -> Ident =
            quiet!{ s:$(!keyword() (!terminator() [_])+) { Ident { id: arena.add(s) } } } / expected!("identifier")

        // Language constructs
        rule expression() -> Expr =
            f:float_literal() { Expr::Lit(Literal::Float(f))} / i:int_literal() { Expr::Lit(Literal::Int(i))} /
            b:bool_literal() { Expr::Lit(Literal::Bool(b))} / s:str_literal() { Expr::Lit(Literal::String(s.to_string()))} /
            i:identifier() { Expr::Ident(i)}
        rule execution() -> Execution =
            i:expression() wsp() m:expression() a:(wsp() e:expression() {e})* { Execution::Called(i, m, a) } /
            i:expression() { Execution::Single(i) }
        rule assignment() -> (Ident, Execution) = lhs:identifier() _() eq() _() rhs:execution() { (lhs, rhs) }
        rule stmt() -> Stmt =
            stmt:(
                a:assignment() { Stmt::Assignment { dst: a.0, rhs: a.1 } } / e:execution() { Stmt::Exe(e) }
            ) _() ";" _() { stmt }

        // Top-level
        pub rule statements() -> Vec<Stmt> = _() stmts:stmt()* _() { stmts }
    }
}
