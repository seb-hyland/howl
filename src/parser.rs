use crate::IdentArena;

pub enum Keyword {
    Type,
}

#[derive(Clone, Copy, Debug)]
pub struct Ident {
    pub id: u64,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assignment { dst: Ident, rhs: Execution },
    Exe(Execution),
}

#[derive(Debug, Clone)]
pub enum Execution {
    Single(Expr),
    Called(Expr, Expr, Vec<Expr>),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Ident(Ident),
    Block(Vec<Stmt>),
    Lit(Literal),
}

#[derive(Clone, Debug)]
pub enum Literal {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Nil,
}

peg::parser! {
    pub grammar howl_parser(arena: &mut IdentArena) for str {
        // Helpers
        rule wsp() = quiet! { [c if c.is_ascii_whitespace()]+ {} }
        rule comment() = "//" (!"\n" [_])* ("\n" / ![_])
        rule _() = quiet! { (wsp() / comment())* }
        rule digit() -> &'input str = quiet! { $[c if c.is_ascii_digit()] } / expected!("digit")
        rule terminator() = ['(' | ')' | '[' | ']' | ',' | ':' | ';' | '|' | '"'] / wsp()
        rule eq() = quiet!{ "=" } / expected!("EQUAL")

        // Atoms
        rule int_literal() -> i32 = n:$(digit()+) { ? n.parse().or(Err("i32")) }
        rule float_literal() -> f64 = n:$(digit()+ "." digit()*) { ? n.parse().or(Err("f64")) }
        rule bool_literal() -> bool = "True" { true } / "False" { false }
        rule str_literal() -> String = "\"" s:("\\" c:['\"' | '\\' | 'n' | 'r' | 't'] { ?
            match c { 'n' => Ok('\n'), 'r' => Ok('\r'), 't' => Ok('\t'), '\\' => Ok('\\'), '"' => Ok('"'),
                _ => Err("Invalid escape character") }} / c:[^ '\"' | '\\'] { c })* "\"" { s.into_iter().collect() }
        rule keyword() -> Keyword = "type" &terminator() { Keyword::Type }
        rule identifier() -> Ident =
            quiet!{ s:$(!keyword() (!terminator() [_])+) { Ident { id: arena.add(s) } } } / expected!("identifier")
        rule block() -> Vec<Stmt> =
            "[" s:statements() "]" { s }

        // Language constructs
        rule expression() -> Expr =
            f:float_literal() { Expr::Lit(Literal::Float(f))} / i:int_literal() { Expr::Lit(Literal::Int(i))} /
            b:bool_literal() { Expr::Lit(Literal::Bool(b))} / s:str_literal() { Expr::Lit(Literal::String(s.to_string()))} /
            "Nil" { Expr::Lit(Literal::Nil) } / i:identifier() { Expr::Ident(i)} / b:block() { Expr::Block(b) }
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
