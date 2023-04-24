#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TokenKind {
    Sc,
    Inc,
    Dec,
    Number(u32),
    Newline,
    LBracket,
    RBracket,
    Comma,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    line: usize,
    col: usize,
}

impl Token {
    pub fn kind(&self) -> TokenKind {
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
}

impl<'a> TokenStream<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(0).cloned()
    }

    fn next_char(&mut self) -> Option<u8> {
        if let ret @ Some(ch) = self.peek() {
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

    fn make_token(&self, kind: TokenKind) -> Token {
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

    fn lex_symbol(&mut self) -> Option<Token> {
        let symbol_tokens = [
            (b'\n', TokenKind::Newline),
            (b'[', TokenKind::LBracket),
            (b']', TokenKind::RBracket),
            (b',', TokenKind::Comma),
        ];

        let next = self.peek()?;

        for (ch, tok) in symbol_tokens {
            if ch == next {
                let ret = self.make_token(tok);
                self.next_char();
                return Some(ret);
            }
        }

        None
    }

    fn lex_keyword(&mut self) -> Option<Token> {
        let mut keywords = [
            (b"inc".as_ref(), TokenKind::Inc),
            (b"dec".as_ref(), TokenKind::Dec),
            (b"sc".as_ref(), TokenKind::Sc),
        ];
        keywords.sort_by_key(|(x, _)| x.len());
        keywords.reverse();

        for (s, tok) in keywords {
            let t = self.make_token(tok);
            if self.eat_string(s) {
                return Some(t);
            }
        }

        None
    }

    fn eat_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.next_char();
        }
    }

    fn lex_number(&mut self) -> Option<Token> {
        let line = self.line;
        let col = self.col;

        let start = self.source;
        let mut num_digits = 0;
        while let Some(b'0'..=b'9') = self.peek() {
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
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let lexers = [Self::lex_symbol, Self::lex_keyword, Self::lex_number];

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
    TokenStream::new(source.as_bytes())
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
    }
}
