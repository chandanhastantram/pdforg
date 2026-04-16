//! Formula parser — converts a token stream into an Expression AST.

use crate::lexer::Token;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use pdf_core::CellAddress;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    Text(String),
    Bool(bool),
    Error(String),
    CellRef(CellAddress),
    RangeRef(CellAddress, CellAddress),
    NamedRange(String),
    Call { name: String, args: Vec<Expr> },
    BinOp { left: Box<Expr>, op: BinOpKind, right: Box<Expr> },
    UnaryOp { op: UnaryOpKind, expr: Box<Expr> },
    Array(Vec<Vec<Expr>>),
    Percent(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinOpKind {
    Add, Sub, Mul, Div, Pow,
    Concat, // &
    Eq, Ne, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOpKind {
    Neg,
    Plus,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token: {0:?}")]
    Unexpected(Token),
    #[error("Unexpected end of input")]
    UnexpectedEOF,
    #[error("Invalid cell reference: {0}")]
    InvalidCellRef(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EOF)
    }

    fn advance(&mut self) -> &Token {
        let tok = self.tokens.get(self.pos).unwrap_or(&Token::EOF);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        let tok = self.advance().clone();
        if std::mem::discriminant(&tok) == std::mem::discriminant(expected) {
            Ok(())
        } else {
            Err(ParseError::Unexpected(tok))
        }
    }

    // Pratt parser with precedence levels
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::Equal => BinOpKind::Eq,
                Token::NotEqual => BinOpKind::Ne,
                Token::LessThan => BinOpKind::Lt,
                Token::LessEqual => BinOpKind::Le,
                Token::GreaterThan => BinOpKind::Gt,
                Token::GreaterEqual => BinOpKind::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_concat()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOpKind::Add,
                Token::Minus => BinOpKind::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_concat()?;
            left = Expr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        while matches!(self.peek(), Token::Ampersand) {
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp { left: Box::new(left), op: BinOpKind::Concat, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOpKind::Mul,
                Token::Slash => BinOpKind::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Expr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        let base = self.parse_unary()?;
        if matches!(self.peek(), Token::Caret) {
            self.advance();
            let exp = self.parse_power()?; // right-associative
            Ok(Expr::BinOp { left: Box::new(base), op: BinOpKind::Pow, right: Box::new(exp) })
        } else {
            Ok(base)
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_postfix()?;
                Ok(Expr::UnaryOp { op: UnaryOpKind::Neg, expr: Box::new(expr) })
            }
            Token::Plus => {
                self.advance();
                let expr = self.parse_postfix()?;
                Ok(Expr::UnaryOp { op: UnaryOpKind::Plus, expr: Box::new(expr) })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        // Postfix percent
        if matches!(self.peek(), Token::Percent) {
            self.advance();
            expr = Expr::Percent(Box::new(expr));
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Token::Number(n) => { self.advance(); Ok(Expr::Number(n)) }
            Token::StringLit(s) => { self.advance(); Ok(Expr::Text(s)) }
            Token::Bool(b) => { self.advance(); Ok(Expr::Bool(b)) }
            Token::Error(e) => { self.advance(); Ok(Expr::Error(e)) }

            Token::CellRef(r) => {
                self.advance();
                let addr = parse_cell_ref(&r)
                    .ok_or_else(|| ParseError::InvalidCellRef(r.clone()))?;
                Ok(Expr::CellRef(addr))
            }

            Token::RangeRef(a, b) => {
                self.advance();
                let start = parse_cell_ref(&a)
                    .ok_or_else(|| ParseError::InvalidCellRef(a.clone()))?;
                let end = parse_cell_ref(&b)
                    .ok_or_else(|| ParseError::InvalidCellRef(b.clone()))?;
                Ok(Expr::RangeRef(start, end))
            }

            Token::Ident(name) => {
                self.advance();
                if matches!(self.peek(), Token::LParen) {
                    // Function call
                    self.advance(); // consume '('
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::NamedRange(name))
                }
            }

            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }

            Token::LBrace => {
                self.advance();
                let array = self.parse_array()?;
                self.expect(&Token::RBrace)?;
                Ok(Expr::Array(array))
            }

            Token::EOF => Err(ParseError::UnexpectedEOF),
            tok => Err(ParseError::Unexpected(tok.clone())),
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = vec![];
        if matches!(self.peek(), Token::RParen) { return Ok(args); }
        args.push(self.parse_expr()?);
        while matches!(self.peek(), Token::Comma | Token::Semicolon) {
            self.advance();
            if matches!(self.peek(), Token::RParen) { break; } // trailing comma
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn parse_array(&mut self) -> Result<Vec<Vec<Expr>>, ParseError> {
        let mut rows = vec![];
        let mut row = vec![];
        loop {
            match self.peek() {
                Token::RBrace => break,
                Token::Semicolon => { self.advance(); rows.push(row); row = vec![]; }
                Token::Comma => { self.advance(); }
                _ => row.push(self.parse_expr()?),
            }
        }
        if !row.is_empty() { rows.push(row); }
        Ok(rows)
    }
}

/// Parse "A1", "$B$2", "Sheet1!C3" into a CellAddress
pub fn parse_cell_ref(s: &str) -> Option<CellAddress> {
    // Handle sheet refs: "Sheet1!A1"
    let (sheet, local) = if let Some(pos) = s.find('!') {
        (Some(s[..pos].to_string()), &s[pos+1..])
    } else {
        (None, s)
    };

    let local = local.trim_matches('$');
    let col_end = local.chars().take_while(|c| c.is_ascii_alphabetic()).count();
    if col_end == 0 { return None; }

    let col_str = &local[..col_end];
    let row_str = local[col_end..].trim_matches('$');

    let col = col_str.chars().fold(0u32, |acc, c| {
        acc * 26 + (c.to_ascii_uppercase() as u32 - 'A' as u32 + 1)
    }).checked_sub(1)?;
    let row: u32 = row_str.parse::<u32>().ok()?.checked_sub(1)?;

    Some(CellAddress { row, col, sheet })
}

/// Parse a formula string into an Expr AST
pub fn parse_formula(formula: &str) -> Result<Expr, String> {
    use crate::lexer::Lexer;
    let mut lex = Lexer::new(formula);
    let tokens = lex.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let expr = parse_formula("=42").unwrap();
        assert!(matches!(expr, Expr::Number(42.0)));
    }

    #[test]
    fn test_parse_sum() {
        let expr = parse_formula("=SUM(A1:B10)").unwrap();
        assert!(matches!(expr, Expr::Call { .. }));
    }

    #[test]
    fn test_parse_if() {
        let expr = parse_formula("=IF(A1>0, \"pos\", \"neg\")").unwrap();
        assert!(matches!(expr, Expr::Call { .. }));
    }

    #[test]
    fn test_parse_binop() {
        let expr = parse_formula("=A1+B2*3").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOpKind::Add, .. }));
    }

    #[test]
    fn test_parse_unary_neg() {
        let expr = parse_formula("=-A1").unwrap();
        assert!(matches!(expr, Expr::UnaryOp { op: UnaryOpKind::Neg, .. }));
    }
}
