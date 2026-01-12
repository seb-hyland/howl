use crate::Span;
use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

pub struct Token {
    pub ty: TokenType,
    pub span: Span,
}

pub enum TokenType {
    // Identifier variants
    Ident(usize),
    /// An ident prefixed by `@` (type-level field or handler)
    TypeIdent(usize),
    /// A path to a value, like `v.buf.0.inner`
    Path(Vec<PathItem>),
    /// A reserved keyword
    Keyword(Keyword),

    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),

    // Punctuation
    Eq,
    At,
    Hashtag,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Colon,
    Comma,
    Pipe,
    Semicolon,
}

pub enum PathItem {
    Ident(usize),
    Int(i64),
}

impl PathItem {
    pub fn emit(&self, arena: &IdentArena, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(i) => write!(f, "Int({i})"),
            Self::Ident(id) => write!(f, "Ident({})", arena.get(*id).unwrap()),
        }
    }
}

pub enum Keyword {
    Type,
    Trait,
    Opaque,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Type => "type",
                Self::Trait => "trait",
                Self::Opaque => "opaque",
            }
        )
    }
}

#[derive(Default)]
pub struct IdentArena {
    map: HashMap<Arc<str>, usize>,
    vec: Vec<Arc<str>>,
}

impl IdentArena {
    pub fn add(&mut self, v: &[u8]) -> usize {
        let s = unsafe { str::from_utf8_unchecked(v) };
        let arc_s = Arc::from(s);

        if let Some(&id) = self.map.get(&arc_s) {
            return id;
        }

        self.vec.push(Arc::clone(&arc_s));
        let id = self.vec.len() - 1;
        self.map.insert(arc_s, id);
        id
    }

    pub fn get(&self, id: usize) -> Option<Arc<str>> {
        self.vec.get(id).map(Arc::clone)
    }
}

impl TokenType {
    pub fn emit(&self, arena: &IdentArena, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Ident(id) => write!(f, "Ident({})", arena.get(*id).unwrap()),
            Self::TypeIdent(id) => write!(f, "TypeIdent({})", arena.get(*id).unwrap()),
            Self::Path(path) => {
                write!(f, "Path(")?;
                let mut l = f.debug_list();
                for seg in path {
                    struct PathItemView<'a>(&'a PathItem, &'a IdentArena);

                    impl Debug for PathItemView<'_> {
                        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                            match self.0 {
                                PathItem::Ident(id) => {
                                    write!(f, "Ident({})", self.1.get(*id).unwrap())
                                }
                                PathItem::Int(i) => write!(f, "Int({i})"),
                            }
                        }
                    }

                    l.entry(&PathItemView(seg, arena));
                }
                l.finish()?;
                write!(f, ")")
            }
            Self::Keyword(kw) => write!(f, "Keyword({kw})"),
            Self::IntLiteral(i) => write!(f, "Int({i})"),
            Self::FloatLiteral(n) => write!(f, "Float({n})"),
            Self::StringLiteral(t) => write!(f, "String({t})"),
            Self::BoolLiteral(b) => write!(f, "Bool({b})"),
            Self::Eq => write!(f, "Eq"),
            Self::At => write!(f, "At"),
            Self::Hashtag => write!(f, "Hashtag"),
            Self::OpenParen => write!(f, "OpenParen"),
            Self::CloseParen => write!(f, "CloseParen"),
            Self::OpenBrace => write!(f, "OpenBrace"),
            Self::CloseBrace => write!(f, "CloseBrace"),
            Self::Colon => write!(f, "Colon"),
            Self::Comma => write!(f, "Comma"),
            Self::Pipe => write!(f, "Pipe"),
            Self::Semicolon => write!(f, "Semicolon"),
        }
    }
}

pub struct TokenPrinter<'a> {
    tokens: &'a Vec<Token>,
    arena: &'a IdentArena,
}

impl<'a> TokenPrinter<'a> {
    pub fn new(v: &'a (Vec<Token>, IdentArena)) -> Self {
        Self {
            tokens: &v.0,
            arena: &v.1,
        }
    }
}

impl<'a> Display for TokenPrinter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for token in self.tokens {
            struct TokenView<'b>(&'b TokenType, &'b IdentArena);

            impl Debug for TokenView<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                    self.0.emit(self.1, f)
                }
            }

            list.entry(&TokenView(&token.ty, self.arena));
        }
        list.finish()
    }
}

type LexResult<T> = Result<T, LexError>;

pub struct LexError {
    err: LexErrorType,
    span: Span,
}

pub enum LexErrorType {
    UnclosedQuote,
    NumberWithMultiplePoints,
    IntParseFailure,
    FloatParseFailure,
    InvalidIdent,
    EmptyPathItem,
}

impl LexError {
    pub fn emit(&self, src: &str) {
        let msg = match self.err {
            LexErrorType::UnclosedQuote => "unclosed string literal",
            LexErrorType::NumberWithMultiplePoints => "number has multiple decimal points",
            LexErrorType::IntParseFailure => "could not parse integer literal",
            LexErrorType::FloatParseFailure => "could not parse float literal",
            LexErrorType::InvalidIdent => "identifiers must not begin with a number or `.`",
            LexErrorType::EmptyPathItem => "data path cannot contain empty segments",
        };
        let snippet = Level::ERROR.primary_title(msg).element(
            Snippet::source(src).annotation(AnnotationKind::Primary.span(self.span.0.clone())),
        );
        let renderer = Renderer::styled();
        println!("{}", renderer.render(&[snippet]));
    }
}

pub struct Lexer<'src> {
    buf: &'src [u8],
    current: usize,
    arena: IdentArena,
    tokens: Vec<Token>,
}

impl<'src> Lexer<'src> {
    pub fn run(src: &'src [u8]) -> LexResult<(Vec<Token>, IdentArena)> {
        let mut parser = Self {
            buf: src,
            current: 0,
            arena: IdentArena::default(),
            tokens: Vec::new(),
        };
        while parser.parse_token()?.is_some() {}
        Ok((parser.tokens, parser.arena))
    }

    fn peek(&self) -> Option<&u8> {
        self.buf.get(self.current)
    }

    fn advance(&mut self) -> Option<&u8> {
        let cur_byte = self.buf.get(self.current);
        if cur_byte.is_some() {
            self.current += 1;
        }
        cur_byte
    }

    fn is_wsp(c: &u8) -> bool {
        matches!(c, b' ' | b'\r' | b'\t' | b'\n')
    }

    fn skip_whitespace(&mut self) {
        while let Some(n) = self.peek()
            && Self::is_wsp(n)
        {
            self.advance();
        }
    }

    fn is_terminator(c: &u8) -> bool {
        Self::is_wsp(c)
            || matches!(
                c,
                b'(' | b')' | b'[' | b']' | b',' | b':' | b';' | b'|' | b'"'
            )
    }

    fn bytes_to_str(bytes: &[u8]) -> &str {
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    fn parse_token(&mut self) -> LexResult<Option<()>> {
        self.skip_whitespace();

        let start = self.current;
        let byte = match self.advance() {
            Some(b) => *b,
            None => return Ok(None),
        };
        let span = Span(start..self.current);

        match byte {
            b'@' => self.tokens.push(Token {
                ty: TokenType::At,
                span,
            }),
            b'#' => self.tokens.push(Token {
                ty: TokenType::Hashtag,
                span,
            }),
            b'(' => self.tokens.push(Token {
                ty: TokenType::OpenParen,
                span,
            }),
            b')' => self.tokens.push(Token {
                ty: TokenType::CloseParen,
                span,
            }),
            b'[' => self.tokens.push(Token {
                ty: TokenType::OpenBrace,
                span,
            }),
            b']' => self.tokens.push(Token {
                ty: TokenType::CloseBrace,
                span,
            }),
            b',' => self.tokens.push(Token {
                ty: TokenType::Comma,
                span,
            }),
            b';' => self.tokens.push(Token {
                ty: TokenType::Semicolon,
                span,
            }),
            b':' => self.tokens.push(Token {
                ty: TokenType::Colon,
                span,
            }),
            b'|' => self.tokens.push(Token {
                ty: TokenType::Pipe,
                span,
            }),
            b'"' => {
                let t = self.quoted(start)?;
                self.tokens.push(t)
            }
            _ => {
                let t = self.text(start)?;
                self.tokens.push(t)
            }
        };
        Ok(Some(()))
    }

    fn quoted(&mut self, start: usize) -> LexResult<Token> {
        while let Some(&n) = self.peek() {
            if n == b'"' {
                self.advance();

                let inner_range = (start + 1)..(self.current - 1);
                return Ok(Token {
                    ty: TokenType::StringLiteral(
                        Self::bytes_to_str(&self.buf[inner_range]).to_owned(),
                    ),
                    span: Span(start..self.current),
                });
            }
            self.advance();
        }

        Err(LexError {
            err: LexErrorType::UnclosedQuote,
            span: Span(start..self.current),
        })
    }

    fn text(&mut self, start: usize) -> LexResult<Token> {
        while let Some(n) = self.peek()
            && !Self::is_terminator(n)
        {
            self.advance();
        }
        let range = start..self.current;
        let bytes = &self.buf[range.clone()];

        assert!(!bytes.is_empty());
        let first_byte = &bytes[0];

        // 0-9 OR .
        fn num_or_point(&b: &u8) -> bool {
            b.is_ascii_digit() || b == b'.'
        }

        // Try to parse as Float or Int literal
        if num_or_point(first_byte) {
            let mut found_point = false;

            for b in bytes {
                if !num_or_point(b) {
                    return Err(LexError {
                        err: LexErrorType::InvalidIdent,
                        span: Span(range),
                    });
                };
                if *b == b'.' {
                    match found_point {
                        false => found_point = true,
                        true => {
                            return Err(LexError {
                                err: LexErrorType::NumberWithMultiplePoints,
                                span: Span(range),
                            });
                        }
                    };
                }
            }

            match found_point {
                false => {
                    let int_s = Self::bytes_to_str(bytes);
                    let int = int_s.parse::<i64>();

                    match int {
                        Ok(i) => {
                            return Ok(Token {
                                ty: TokenType::IntLiteral(i),
                                span: Span(range),
                            });
                        }
                        Err(_) => {
                            return Err(LexError {
                                err: LexErrorType::IntParseFailure,
                                span: Span(range),
                            });
                        }
                    };
                }
                true => {
                    let float_s = Self::bytes_to_str(bytes);
                    let float = float_s.parse::<f64>();

                    match float {
                        Ok(i) => {
                            return Ok(Token {
                                ty: TokenType::FloatLiteral(i),
                                span: Span(range),
                            });
                        }
                        Err(_) => {
                            return Err(LexError {
                                err: LexErrorType::FloatParseFailure,
                                span: Span(range),
                            });
                        }
                    };
                }
            }
        }

        // Try to parse as boolean literal OR reserved keyword
        let bool_or_kw_ty = match bytes {
            b"true" => Some(TokenType::BoolLiteral(true)),
            b"false" => Some(TokenType::BoolLiteral(false)),
            b"type" => Some(TokenType::Keyword(Keyword::Type)),
            b"trait" => Some(TokenType::Keyword(Keyword::Trait)),
            b"opaque" => Some(TokenType::Keyword(Keyword::Trait)),
            _ => None,
        };
        if let Some(ty) = bool_or_kw_ty {
            return Ok(Token {
                ty,
                span: Span(range),
            });
        }

        // Try to parse as Eq
        if bytes == b"=" {
            return Ok(Token {
                ty: TokenType::Eq,
                span: Span(range),
            });
        }

        // Try to parse as TypeIdent
        if bytes[0] == b'@' {
            let id = self.arena.add(bytes);
            return Ok(Token {
                ty: TokenType::TypeIdent(id),
                span: Span(range),
            });
        }

        // Try to parse as path
        if bytes.contains(&b'.') {
            let mut last_start = 0;
            let mut path = Vec::new();

            for i in 0..bytes.len() {
                let b = bytes[i];
                let at_end = i == bytes.len() - 1;

                if b == b'.' || at_end {
                    if last_start == i {
                        return Err(LexError {
                            err: LexErrorType::EmptyPathItem,
                            span: Span(range),
                        });
                    }

                    let chunk = if at_end {
                        &bytes[last_start..=i]
                    } else {
                        &bytes[last_start..i]
                    };

                    let first_byte = chunk[0];
                    if first_byte.is_ascii_digit() {
                        let int = Self::bytes_to_str(chunk).parse::<i64>().map_err(|_| {
                            let chunk_start = start + i;
                            LexError {
                                err: LexErrorType::InvalidIdent,
                                span: Span(chunk_start..(chunk_start + chunk.len())),
                            }
                        })?;
                        path.push(PathItem::Int(int));
                    } else {
                        let id = self.arena.add(chunk);
                        path.push(PathItem::Ident(id));
                    }

                    last_start = i + 1;
                }
            }

            return Ok(Token {
                ty: TokenType::Path(path),
                span: Span(range),
            });
        }

        // I guess it's just a normal ident 😑
        let id = self.arena.add(bytes);
        Ok(Token {
            ty: TokenType::Ident(id),
            span: Span(range),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_and_print(input: &str) {
        println!("Input: {}", input);
        match Lexer::run(input.as_bytes()) {
            Ok(out) => println!("Tokenized: {:#}", TokenPrinter::new(&out)),
            Err(e) => e.emit(input),
        }
    }

    #[test]
    fn test_lex_tuple() {
        let input = r#"v: (Float, Int) = (3, 4.15);"#;
        lex_and_print(input);
    }

    #[test]
    fn test_lex_tuple_with_string() {
        let input = r#"v.buf.0.inner: (Float, String, Float) = (3.2, "This is a string with whitespace", 4.);"#;
        lex_and_print(input);
    }

    #[test]
    fn test_lex_unclosed_delim() {
        let input = r#"v: (Float, String, Float) = (3.2, "This is an unclosed string, 4.);"#;
        lex_and_print(input);
    }

    #[test]
    fn test_lex_invalid_ident() {
        let input = r#"
            w = (3, 4.15);
            .v: (Float, Int) = (3, 4.15);
        "#;
        lex_and_print(input);
    }
}
