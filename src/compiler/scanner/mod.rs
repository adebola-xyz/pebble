mod helpers;
mod token;
use helpers::*;
use token::{
    BracketKind::*,
    Comment, CommentKind,
    NumericKind::*,
    StringInnerToken::{self, *},
    Token::{self, *},
};

#[derive(Debug)]
pub struct ScannerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

type ScanResult = Result<Vec<Token>, ScannerError>;
type ScanInternalResult = Result<(), ScannerError>;

pub fn scan(content: String) -> ScanResult {
    let content: Vec<char> = content.chars().collect();
    let mut scanner = Scanner::new();
    scanner.scan(content)?;
    Ok(scanner.tokens)
}

struct Scanner {
    source: Vec<char>,
    /// The current character in the stream.
    current: char,
    tokens: Vec<Token>,
    /// The tracker for the lines and columns in the source text.
    pos: [usize; 2],
    index: usize,
    end: bool,
    store: [usize; 4],
    comments: Vec<Comment>,
}

impl Scanner {
    fn new() -> Self {
        Scanner {
            source: vec![],
            tokens: vec![],
            comments: vec![],
            current: '\0',
            pos: [1, 0],
            index: 0,
            store: [0, 0, 0, 0],
            end: false,
        }
    }
    /// Run the scanner.
    fn scan(&mut self, content: Vec<char>) -> Result<(), ScannerError> {
        self.source = content;
        self.set();
        while !self.end {
            if self.expects("//") {
                self.scan_line_comment()?
            } else if self.expects("/*") {
                self.scan_block_comment()?
            } else if self.current.is_digit(10) {
                self.scan_number()?
            } else {
                match self.current {
                    '"' => self.scan_string()?,
                    '{' | '}' | '[' | ']' | '(' | ')' => self.scan_brackets()?,
                    '@' => self.scan_injunction()?,
                    _ => {}
                }
            }
            self.next();
        }
        Ok(())
    }
    /// Set the scanner to the positions to start scanning.
    fn set(&mut self) {
        if let Some(c) = self.source.get(self.index) {
            self.current = *c;
        } else {
            self.end = true;
        };
        self.shift();
    }
    /// Advance to the next character in the stream.
    fn next(&mut self) {
        self.index += 1;
        if self.index >= self.source.len() {
            self.end = true;
            self.current = '\0'
        } else {
            self.current = self.source[self.index];
            self.shift();
        }
    }
    /// Advance by a particular length.
    fn next_by(&mut self, l: usize) {
        for _ in 0..l {
            self.next();
        }
    }
    /// Shift the line tracker.
    fn shift(&mut self) {
        if self.current == '\n' {
            self.pos[0] += 1;
            self.pos[1] = 0;
        } else {
            self.pos[1] += 1;
        }
    }
    fn _lookahead(&self, i: usize) -> Option<char> {
        if self.index + i >= self.source.len() {
            None
        } else {
            Some(self.source[self.index + i])
        }
    }
    fn expects(&self, pattern: &str) -> bool {
        let end = self.index + pattern.len();
        if end > self.source.len() {
            false
        } else {
            let section: String = self.source[self.index..end].iter().collect();
            section == pattern.to_string()
        }
    }
    /// Emits an error encountered during scanning.
    fn error(&self, message: &str) -> ScanInternalResult {
        Err(ScannerError {
            message: message.to_string(),
            line: self.pos[0],
            column: self.pos[1],
        })
    }
    /// Takes a snapshot of the position of the scanner at a point during the scanning.
    fn loc_start(&mut self) {
        self.store[0] = self.pos[0];
        self.store[1] = self.pos[1];
    }
    /// Takes a snapshot of the position of the scanner at a point during the scanning.
    fn loc_end(&mut self) {
        self.store[2] = self.pos[0];
        self.store[3] = self.pos[1];
    }
    fn scan_block_comment(&mut self) -> ScanInternalResult {
        self.loc_start();
        self.next_by(2);
        let mut value = String::new();
        while !(self.end || self.expects("*/")) {
            value.push(self.current);
            self.next()
        }
        if self.end {
            self.error("Unclosed block comment.")?
        }
        self.next_by(2);
        self.loc_end();
        self.comments.push(Comment {
            kind: CommentKind::Block,
            value,
            loc: self.store,
        });
        Ok(())
    }
    fn scan_line_comment(&mut self) -> ScanInternalResult {
        self.loc_start();
        self.next_by(2);
        let mut value = String::new();
        while !(self.end || self.current == '\n') {
            value.push(self.current);
            self.next();
        }
        if !self.end {
            self.next();
        }
        self.loc_end();
        self.comments.push(Comment {
            kind: CommentKind::Line,
            value,
            loc: self.store,
        });
        Ok(())
    }
    fn scan_brackets(&mut self) -> ScanInternalResult {
        self.loc_start();
        let kind;
        match self.current {
            '{' => kind = LCurly,
            '}' => kind = RCurly,
            '(' => kind = LParen,
            ')' => kind = RParen,
            '[' => kind = LSquare,
            ']' => kind = RSquare,
            _ => panic!(),
        }
        self.next();
        self.loc_end();
        let token = Bracket {
            kind,
            loc: self.store,
        };
        self.tokens.push(token);
        Ok(())
    }
    /// Tokenize a number.
    fn scan_number(&mut self) -> ScanInternalResult {
        self.loc_start();
        let raw = self.count(10);
        if self.current.is_alphabetic() {
            self.error("An identifier cannot immediately follow a literal.")?
        };
        self.loc_end();
        let token = Number {
            kind: Decimal,
            raw,
            loc: self.store,
        };
        self.tokens.push(token);
        Ok(())
    }
    /// Counts all the succeeding characters in the text stream that are numbers.
    fn count(&mut self, base: usize) -> String {
        let mut num = String::new();
        while self.current.is_digit(base as u32) {
            num.push(self.current);
            self.next();
        }
        num
    }
    fn scan_injunction(&mut self) -> ScanInternalResult {
        self.loc_start();
        self.next();
        if !is_identifier_char(self.current) || self.current.is_digit(10) {
            self.error("The scannner expected an identifier character.")?
        }
        let mut value = String::new();
        while !self.end && is_identifier_char(self.current) {
            value.push(self.current);
            self.next();
        }
        self.loc_end();
        if !is_injunction_value(&value) {
            let message = format!("Unrecognized injunction \"{}\".", value);
            self.error(message.as_str())?
        }
        self.tokens.push(Injunction {
            value,
            loc: self.store,
        });
        Ok(())
    }
    fn scan_string(&mut self) -> ScanInternalResult {
        self.loc_start();
        self.next();
        let mut inner = vec![];
        while !(self.end || self.expects("\"")) {
            if self.expects("#{") {
                self.next_by(2);
                inner.push(self.scan_string_expression()?);
                self.next();
            } else {
                inner.push(self.scan_string_sequence()?);
                self.next();
            }
        }
        if self.end {
            self.error("Unterminated string literal.")?;
        }
        self.next();
        self.loc_end();
        self.tokens.push(StringToken {
            inner,
            loc: self.store,
        });
        Ok(())
    }
    fn scan_string_sequence(&mut self) -> Result<StringInnerToken, ScannerError> {
        let start = self.pos.clone();
        let mut value = String::new();
        while !(self.end || self.expects("\"")) {
            value.push(self.current);
            if let Some('#') = self._lookahead(1) {
                if let Some('{') = self._lookahead(2) {
                    break;
                }
            }
            if let Some('\"') = self._lookahead(1) {
                break;
            } else {
                self.next();
            }
        }
        let end = self.pos.clone();
        let loc = [start[0], start[1], end[0], end[1]];
        Ok(StringSequence { value, loc })
    }
    fn scan_string_expression(&mut self) -> Result<StringInnerToken, ScannerError> {
        let start = self.pos.clone();
        let end = self.pos.clone();
        let loc = [start[0], start[1], end[0], end[1]];
        Ok(StringExpression {
            tokens: vec![],
            loc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_scans_injunction_token() {
        let tokens = scan("@public ".to_string()).unwrap();
        assert_eq!(
            tokens[0],
            Injunction {
                value: String::from("public"),
                loc: [1, 1, 1, 8]
            }
        );
    }
    #[test]
    fn it_scans_string_token() {
        let tokens = scan("\"Hello from the other side.\"".to_string()).unwrap();
        assert_eq!(
            tokens[0],
            StringToken {
                inner: vec![StringSequence {
                    loc: [1, 2, 1, 27],
                    value: String::from("Hello from the other side.")
                }],
                loc: [1, 1, 1, 28]
            }
        )
    }
    #[test]
    fn it_scans_number() {
        let tokens = scan("99923".to_string()).unwrap();
        assert_eq!(
            tokens[0],
            Number {
                kind: Decimal,
                raw: String::from("99923"),
                loc: [1, 1, 1, 5]
            }
        )
    }
    #[test]
    fn it_scans_brackets() {
        let tokens = scan("() @variable".to_string()).unwrap();
        assert_eq!(
            tokens[0],
            Bracket {
                kind: LParen,
                loc: [1, 1, 1, 2]
            }
        )
    }
}
