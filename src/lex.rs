#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TokenKind<'a> {
    Ch,
    Tch,
    Sc,
    Fpsc,
    Bpsc,
    Blsc,
    Inc,
    Flinc,
    Blinc,
    Dec,
    InMr,
    Number(u32),
    Newline,
    LBracket,
    RBracket,
    Comma,
    Comment(&'a str),
    Skip,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Token<'a> {
    kind: TokenKind<'a>,
    line: usize,
    col: usize,
}

impl<'a> Token<'a> {
    pub fn kind(&self) -> TokenKind<'a> {
        self.kind
    }

    pub fn source_loc(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}

#[derive(Debug)]
pub struct TokenStream<'a> {
    source: &'a [u8],
    line: usize,
    col: usize,
    peeked_token: Option<Token<'a>>,
}

impl TokenStream<'_> {
    pub fn current_loc(&self) -> (usize, usize) {
        match &self.peeked_token {
            Some(p) => p.source_loc(),
            None => (self.line, self.col),
        }
    }
}

impl<'a> TokenStream<'a> {
    pub fn peek(&mut self) -> Option<Token<'a>> {
        if self.peeked_token.is_none() {
            self.peeked_token = self.next();
        }
        self.peeked_token
    }

    pub fn peek_kind(&mut self) -> Option<TokenKind<'a>> {
        self.peek().map(|x| x.kind())
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty() && self.peeked_token.is_none()
    }

    fn new(source: &'a str) -> Self {
        Self {
            source: source.trim_end().as_bytes(),
            line: 1,
            col: 1,
            peeked_token: None,
        }
    }

    fn peek_char(&self) -> Option<u8> {
        self.source.get(0).cloned()
    }

    fn next_char(&mut self) -> Option<u8> {
        if let ret @ Some(ch) = self.peek_char() {
            if ch == b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }

            self.source = &self.source[1..];

            ret
        } else {
            None
        }
    }

    fn make_token<'x>(&self, kind: TokenKind<'x>) -> Token<'x> {
        Token {
            kind,
            line: self.line,
            col: self.col,
        }
    }

    fn eat_string(&mut self, string: &[u8]) -> bool {
        if self.source.starts_with(string) {
            for _ in 0..string.len() {
                self.next_char();
            }

            true
        } else {
            false
        }
    }

    fn lex_symbol(&mut self) -> Option<Token<'a>> {
        let symbol_tokens = [
            (b'\n', TokenKind::Newline),
            (b'[', TokenKind::LBracket),
            (b']', TokenKind::RBracket),
            (b',', TokenKind::Comma),
        ];

        let next = self.peek_char()?;

        for (ch, tok) in symbol_tokens {
            if ch == next {
                let ret = self.make_token(tok);
                self.next_char();
                return Some(ret);
            }
        }

        None
    }

    fn lex_keyword(&mut self) -> Option<Token<'a>> {
        let mut keywords = [
            (b"in mr".as_ref(), TokenKind::InMr),
            (b"blinc".as_ref(), TokenKind::Blinc),
            (b"flinc".as_ref(), TokenKind::Flinc),
            (b"fpsc".as_ref(), TokenKind::Fpsc),
            (b"bpsc".as_ref(), TokenKind::Bpsc),
            (b"blsc".as_ref(), TokenKind::Blsc),
            (b"inc".as_ref(), TokenKind::Inc),
            (b"dec".as_ref(), TokenKind::Dec),
            (b"sc".as_ref(), TokenKind::Sc),
            (b"ch".as_ref(), TokenKind::Ch),
            (b"tch".as_ref(), TokenKind::Tch),
            (b"skip".as_ref(), TokenKind::Skip),
        ];
        keywords.sort_by_key(|(x, _)| std::cmp::Reverse(x.len()));

        for (s, tok) in keywords {
            let t = self.make_token(tok);
            if self.eat_string(s) {
                return Some(t);
            }
        }

        None
    }

    fn eat_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(b' ' | b'\t')) {
            self.next_char();
        }
    }

    fn lex_number(&mut self) -> Option<Token<'a>> {
        let line = self.line;
        let col = self.col;

        let start = self.source;
        let mut num_digits = 0;
        while let Some(b'0'..=b'9') = self.peek_char() {
            self.next_char();
            num_digits += 1;
        }

        if num_digits == 0 {
            None
        } else {
            Some(Token {
                kind: TokenKind::Number(
                    std::str::from_utf8(&start[..num_digits])
                        .unwrap()
                        .parse()
                        .unwrap(),
                ),
                line,
                col,
            })
        }
    }

    fn lex_comment(&mut self) -> Option<Token<'a>> {
        if let Some(b'%') = self.peek_char() {
            let line = self.line;
            let col = self.col;
            let source_before_comment = self.source;

            self.next_char();
            let beginning = self.source;
            let mut idx = 0;
            let mut closed = false;
            while let Some(ch) = self.next_char() {
                if ch == b'%' {
                    closed = true;
                    break;
                }
                idx += 1;
            }

            if !closed {
                self.source = source_before_comment;
                self.line = line;
                self.col = col;
                return None;
            }

            Some(Token {
                kind: TokenKind::Comment(
                    std::str::from_utf8(&beginning[..idx])
                        .unwrap()
                        .trim()
                        .into(),
                ),
                line,
                col,
            })
        } else {
            None
        }
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked_token.is_some() {
            return self.peeked_token.take();
        }

        let lexers = [
            Self::lex_symbol,
            Self::lex_keyword,
            Self::lex_number,
            Self::lex_comment,
        ];

        self.eat_whitespace();

        for l in lexers {
            if let ret @ Some(_) = l(self) {
                return ret;
            }
        }

        None
    }
}

pub fn tokenize<'a>(source: &'a str) -> TokenStream<'a> {
    TokenStream::new(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        use TokenKind::*;

        let src = "sc 6\ninc 6\nsc 2, [sc, inc] 5";

        let expected = vec![
            Token {
                kind: Sc,
                line: 1,
                col: 1,
            },
            Token {
                kind: Number(6),
                line: 1,
                col: 4,
            },
            Token {
                kind: Newline,
                line: 1,
                col: 5,
            },
            Token {
                kind: Inc,
                line: 2,
                col: 1,
            },
            Token {
                kind: Number(6),
                line: 2,
                col: 5,
            },
            Token {
                kind: Newline,
                line: 2,
                col: 6,
            },
            Token {
                kind: Sc,
                line: 3,
                col: 1,
            },
            Token {
                kind: Number(2),
                line: 3,
                col: 4,
            },
            Token {
                kind: Comma,
                line: 3,
                col: 5,
            },
            Token {
                kind: LBracket,
                line: 3,
                col: 7,
            },
            Token {
                kind: Sc,
                line: 3,
                col: 8,
            },
            Token {
                kind: Comma,
                line: 3,
                col: 10,
            },
            Token {
                kind: Inc,
                line: 3,
                col: 12,
            },
            Token {
                kind: RBracket,
                line: 3,
                col: 15,
            },
            Token {
                kind: Number(5),
                line: 3,
                col: 17,
            },
        ];

        assert_eq!(tokenize(&src).collect::<Vec<_>>(), expected);

        assert_eq!(
            tokenize("% hello there %").collect::<Vec<_>>(),
            vec![Token {
                kind: Comment("hello there".into()),
                line: 1,
                col: 1
            }]
        );
    }
}
