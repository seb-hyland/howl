use crate::Span;
use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use std::fmt::Debug;

pub struct Token<'src> {
    pub ty: TokenType<'src>,
    pub span: Span,
}

impl<'src> Debug for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.ty)
    }
}

pub enum TokenType<'src> {
    Ident(&'src [u8]),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(&'src [u8]),
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

impl<'src> Debug for TokenType<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ident(t) => format!("Ident({})", String::from_utf8_lossy(t)),
                Self::IntLiteral(i) => format!("Int({i})"),
                Self::FloatLiteral(f) => format!("Float({f})"),
                Self::StringLiteral(t) => format!("String({})", String::from_utf8_lossy(t)),
                Self::Eq => "Eq".to_owned(),
                Self::At => "At".to_owned(),
                Self::Hashtag => "Hashtag".to_owned(),
                Self::OpenParen => "OpenParen".to_owned(),
                Self::CloseParen => "CloseParen".to_owned(),
                Self::OpenBrace => "OpenBrace".to_owned(),
                Self::CloseBrace => "CloseBrace".to_owned(),
                Self::Colon => "Colon".to_owned(),
                Self::Comma => "Comma".to_owned(),
                Self::Pipe => "Pipe".to_owned(),
                Self::Semicolon => "Semicolon".to_owned(),
            }
        )
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
}

impl LexError {
    pub fn emit(&self, src: &str) {
        let msg = match self.err {
            LexErrorType::UnclosedQuote => "unclosed string literal",
            LexErrorType::NumberWithMultiplePoints => "number has multiple decimal points",
            LexErrorType::IntParseFailure => "could not parse integer literal",
            LexErrorType::FloatParseFailure => "could not parse float literal",
            LexErrorType::InvalidIdent => "identifiers must not begin with a number or `.`",
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
    tokens: Vec<Token<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn run(src: &'src [u8]) -> LexResult<Vec<Token<'src>>> {
        let mut parser = Self {
            buf: src,
            current: 0,
            tokens: Vec::new(),
        };
        while parser.parse_token()?.is_some() {}
        Ok(parser.tokens)
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
            b'"' => self.quoted(start)?,
            _ => self.text(start)?,
        };
        Ok(Some(()))
    }

    fn quoted(&mut self, start: usize) -> LexResult<()> {
        while let Some(&n) = self.peek() {
            if n == b'"' {
                self.advance();

                let inner_range = (start + 1)..(self.current - 1);
                self.tokens.push(Token {
                    ty: TokenType::StringLiteral(&self.buf[inner_range]),
                    span: Span(start..self.current),
                });

                return Ok(());
            }
            self.advance();
        }

        Err(LexError {
            err: LexErrorType::UnclosedQuote,
            span: Span(start..self.current),
        })
    }

    fn text(&mut self, start: usize) -> LexResult<()> {
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
            (48..=57).contains(&b) || b == b'.'
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
                    let int_s = unsafe { str::from_utf8_unchecked(bytes) };
                    let int = int_s.parse::<i64>();

                    match int {
                        Ok(i) => self.tokens.push(Token {
                            ty: TokenType::IntLiteral(i),
                            span: Span(range),
                        }),
                        Err(_) => {
                            return Err(LexError {
                                err: LexErrorType::IntParseFailure,
                                span: Span(range),
                            });
                        }
                    };
                }
                true => {
                    let float_s = unsafe { str::from_utf8_unchecked(bytes) };
                    let float = float_s.parse::<f64>();

                    match float {
                        Ok(i) => self.tokens.push(Token {
                            ty: TokenType::FloatLiteral(i),
                            span: Span(range),
                        }),
                        Err(_) => {
                            return Err(LexError {
                                err: LexErrorType::FloatParseFailure,
                                span: Span(range),
                            });
                        }
                    };
                }
            }
        } else {
            let ty = if bytes == b"=" {
                TokenType::Eq
            } else {
                TokenType::Ident(bytes)
            };
            self.tokens.push(Token {
                ty,
                span: Span(range),
            });
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_tuple() {
        let def = r#"v: (Float, Int) = (3, 4.15);"#;
        println!("Input: {}", def);
        match Lexer::run(def.as_bytes()) {
            Ok(tt) => println!("Tokenized: {tt:#?}"),
            Err(e) => e.emit(def),
        }
    }

    #[test]
    fn test_lex_tuple_with_string() {
        let def = r#"v: (Float, String, Float) = (3.2, "This is a string with whitespace", 4.);"#;
        println!("Input: {}", def);
        match Lexer::run(def.as_bytes()) {
            Ok(tt) => println!("Tokenized: {tt:#?}"),
            Err(e) => e.emit(def),
        }
    }

    #[test]
    fn test_lex_unclosed_delim() {
        let def = r#"v: (Float, String, Float) = (3.2, "This is an unclosed string, 4.);"#;
        println!("Input: {}", def);
        match Lexer::run(def.as_bytes()) {
            Ok(tt) => println!("Tokenized: {tt:#?}"),
            Err(e) => e.emit(def),
        }
    }

    #[test]
    fn test_lex_invalid_ident() {
        let def = r#"
            w = (3, 4.15);
            .v: (Float, Int) = (3, 4.15);
        "#;
        println!("Input: {}", def);
        match Lexer::run(def.as_bytes()) {
            Ok(tt) => println!("Tokenized: {tt:#?}"),
            Err(e) => e.emit(def),
        }
    }
}
