//! Formula lexer — tokenizes formula strings like "=SUM(A1:B10, 3.14)"

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    StringLit(String),
    Bool(bool),
    Error(String),    // #DIV/0!, #N/A, etc.

    // Identifiers and references
    Ident(String),    // function names, named ranges
    CellRef(String),  // A1, $B$2, Sheet1!C3
    RangeRef(String, String),  // A1:B10

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Caret,           // ^ (power)
    Ampersand,       // & (concatenate)
    Percent,         // % (percentage as postfix)
    Equal,
    NotEqual,        // <>
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,

    // Punctuation
    LParen,
    RParen,
    LBrace,          // { for array literals
    RBrace,          // }
    Comma,
    Semicolon,       // sometimes used as argument separator
    Colon,
    Dollar,
    Exclamation,     // ! for sheet references

    // Special
    EOF,
}

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),
    #[error("Unterminated string literal")]
    UnterminatedString,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        // Skip leading '='
        let s = input.trim();
        let chars: Vec<char> = if s.starts_with('=') {
            s[1..].chars().collect()
        } else {
            s.chars().collect()
        };
        Lexer { input: chars, pos: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = vec![];
        loop {
            let tok = self.next_token()?;
            let done = tok == Token::EOF;
            tokens.push(tok);
            if done { break; }
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> { self.input.get(self.pos).copied() }
    fn peek2(&self) -> Option<char> { self.input.get(self.pos + 1).copied() }
    fn advance(&mut self) -> Option<char> {
        let c = self.input.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        match self.peek() {
            None => Ok(Token::EOF),
            Some(c) => match c {
                '+' => { self.advance(); Ok(Token::Plus) }
                '-' => { self.advance(); Ok(Token::Minus) }
                '*' => { self.advance(); Ok(Token::Star) }
                '/' => { self.advance(); Ok(Token::Slash) }
                '^' => { self.advance(); Ok(Token::Caret) }
                '&' => { self.advance(); Ok(Token::Ampersand) }
                '%' => { self.advance(); Ok(Token::Percent) }
                '(' => { self.advance(); Ok(Token::LParen) }
                ')' => { self.advance(); Ok(Token::RParen) }
                '{' => { self.advance(); Ok(Token::LBrace) }
                '}' => { self.advance(); Ok(Token::RBrace) }
                ',' => { self.advance(); Ok(Token::Comma) }
                ';' => { self.advance(); Ok(Token::Semicolon) }
                '!' => { self.advance(); Ok(Token::Exclamation) }
                '$' => { self.advance(); Ok(Token::Dollar) }

                '=' => { self.advance(); Ok(Token::Equal) }
                '<' => {
                    self.advance();
                    if self.peek() == Some('>') { self.advance(); Ok(Token::NotEqual) }
                    else if self.peek() == Some('=') { self.advance(); Ok(Token::LessEqual) }
                    else { Ok(Token::LessThan) }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); Ok(Token::GreaterEqual) }
                    else { Ok(Token::GreaterThan) }
                }

                '"' => self.lex_string(),
                '#' => self.lex_error_val(),

                '0'..='9' | '.' => self.lex_number(),
                'A'..='Z' | 'a'..='z' | '_' => self.lex_ident_or_cellref(),
                ':' => { self.advance(); Ok(Token::Colon) }

                other => {
                    let pos = self.pos;
                    self.advance();
                    Err(LexError::UnexpectedChar(other, pos))
                }
            }
        }
    }

    fn lex_string(&mut self) -> Result<Token, LexError> {
        self.advance(); // consume opening "
        let mut s = String::new();
        loop {
            match self.advance() {
                None => return Err(LexError::UnterminatedString),
                Some('"') => {
                    // Handle escaped quote: ""
                    if self.peek() == Some('"') {
                        self.advance();
                        s.push('"');
                    } else {
                        break;
                    }
                }
                Some(c) => s.push(c),
            }
        }
        Ok(Token::StringLit(s))
    }

    fn lex_error_val(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if !c.is_whitespace() && c != ')' && c != ',') {
            self.advance();
        }
        let s: String = self.input[start..self.pos].iter().collect();
        Ok(Token::Error(s))
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while matches!(self.peek(), Some('0'..='9' | '.')) {
            self.advance();
        }
        // Scientific notation: 1e10, 1E-5
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) { self.advance(); }
            while matches!(self.peek(), Some('0'..='9')) { self.advance(); }
        }
        let s: String = self.input[start..self.pos].iter().collect();
        let n: f64 = s.parse().unwrap_or(0.0);
        Ok(Token::Number(n))
    }

    fn lex_ident_or_cellref(&mut self) -> Result<Token, LexError> {
        let start = self.pos;

        // Cell references can start with $, so handle here
        let has_dollar_col = self.peek() == Some('$');
        if has_dollar_col { self.advance(); }

        // Read column letters
        let col_start = self.pos;
        while matches!(self.peek(), Some('A'..='Z' | 'a'..='z')) {
            self.advance();
        }
        let col_part: String = self.input[col_start..self.pos].iter().collect();

        // Check for optional $ before row number
        let has_dollar_row = self.peek() == Some('$');
        if has_dollar_row { self.advance(); }

        // Check if followed by row digits
        let row_start = self.pos;
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        let row_part: String = self.input[row_start..self.pos].iter().collect();

        if !col_part.is_empty() && !row_part.is_empty() {
            // It's a cell reference like A1, $B$3
            let cell_ref: String = self.input[start..self.pos].iter().collect();

            // Check if followed by : for range
            if self.peek() == Some(':') {
                self.advance(); // consume ':'

                // Parse second cell ref
                let start2 = self.pos;
                if self.peek() == Some('$') { self.advance(); }
                while matches!(self.peek(), Some('A'..='Z' | 'a'..='z')) { self.advance(); }
                if self.peek() == Some('$') { self.advance(); }
                while matches!(self.peek(), Some('0'..='9')) { self.advance(); }
                let cell_ref2: String = self.input[start2..self.pos].iter().collect();
                return Ok(Token::RangeRef(cell_ref, cell_ref2));
            }
            return Ok(Token::CellRef(cell_ref));
        }

        // Also consume more chars for identifiers (function names, named ranges)
        while matches!(self.peek(), Some('A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '.')) {
            self.advance();
        }
        let ident: String = self.input[start..self.pos].iter().collect();

        // Check for sheet reference: Sheet1!A1
        if self.peek() == Some('!') {
            self.advance();
            let ref_start = self.pos;
            // Read rest of cell ref
            if self.peek() == Some('$') { self.advance(); }
            while matches!(self.peek(), Some('A'..='Z' | 'a'..='z')) { self.advance(); }
            if self.peek() == Some('$') { self.advance(); }
            while matches!(self.peek(), Some('0'..='9')) { self.advance(); }
            let cell_part: String = self.input[ref_start..self.pos].iter().collect();
            return Ok(Token::CellRef(format!("{}!{}", ident, cell_part)));
        }

        // Check for boolean keywords
        match ident.to_uppercase().as_str() {
            "TRUE" => return Ok(Token::Bool(true)),
            "FALSE" => return Ok(Token::Bool(false)),
            _ => {}
        }

        Ok(Token::Ident(ident))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_sum() {
        let mut lex = Lexer::new("=SUM(A1:B10)");
        let tokens = lex.tokenize().unwrap();
        assert!(tokens.contains(&Token::Ident("SUM".into())));
    }

    #[test]
    fn test_lex_number() {
        let mut lex = Lexer::new("=3.14");
        let tokens = lex.tokenize().unwrap();
        assert!(tokens.contains(&Token::Number(3.14)));
    }

    #[test]
    fn test_lex_string() {
        let mut lex = Lexer::new(r#"="Hello World""#);
        let tokens = lex.tokenize().unwrap();
        assert!(tokens.contains(&Token::StringLit("Hello World".into())));
    }

    #[test]
    fn test_lex_range() {
        let mut lex = Lexer::new("=A1:B10");
        let tokens = lex.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::RangeRef(..)));
    }
}
