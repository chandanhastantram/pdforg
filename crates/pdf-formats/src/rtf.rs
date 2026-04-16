//! RTF parser — tokenizes and parses RTF documents.

use crate::FormatError;
use pdf_core::document::{Document, Block, Paragraph, Run, RunFormat};

/// RTF token types
#[derive(Debug, Clone, PartialEq)]
enum RtfToken {
    GroupStart,
    GroupEnd,
    ControlWord(String, Option<i32>),
    ControlSymbol(char),
    Text(String),
}

struct RtfLexer {
    input: Vec<char>,
    pos: usize,
}

impl RtfLexer {
    fn new(input: &str) -> Self {
        RtfLexer { input: input.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> { self.input.get(self.pos).copied() }
    fn advance(&mut self) -> Option<char> {
        let c = self.input.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn tokenize(&mut self) -> Vec<RtfToken> {
        let mut tokens = vec![];
        while let Some(c) = self.peek() {
            match c {
                '{' => { self.advance(); tokens.push(RtfToken::GroupStart); }
                '}' => { self.advance(); tokens.push(RtfToken::GroupEnd); }
                '\\' => {
                    self.advance();
                    match self.peek() {
                        None => break,
                        Some(c) if c.is_ascii_alphabetic() => {
                            let word = self.read_control_word();
                            let param = self.read_int_param();
                            tokens.push(RtfToken::ControlWord(word, param));
                            // Skip optional trailing space
                            if self.peek() == Some(' ') { self.advance(); }
                        }
                        Some(c) => {
                            let sym = c;
                            self.advance();
                            tokens.push(RtfToken::ControlSymbol(sym));
                        }
                    }
                }
                '\r' | '\n' => { self.advance(); }
                _ => {
                    let text = self.read_text();
                    if !text.is_empty() {
                        tokens.push(RtfToken::Text(text));
                    }
                }
            }
        }
        tokens
    }

    fn read_control_word(&mut self) -> String {
        let mut word = String::new();
        while matches!(self.peek(), Some(c) if c.is_ascii_alphabetic()) {
            word.push(self.advance().unwrap());
        }
        word
    }

    fn read_int_param(&mut self) -> Option<i32> {
        let negative = self.peek() == Some('-');
        if negative { self.advance(); }
        let mut digits = String::new();
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            digits.push(self.advance().unwrap());
        }
        if digits.is_empty() { return None; }
        let n: i32 = digits.parse().ok()?;
        Some(if negative { -n } else { n })
    }

    fn read_text(&mut self) -> String {
        let mut text = String::new();
        while matches!(self.peek(), Some(c) if c != '{' && c != '}' && c != '\\' && c != '\r' && c != '\n') {
            text.push(self.advance().unwrap());
        }
        text
    }
}

/// Parse RTF bytes into a Document
pub fn parse_rtf(bytes: &[u8]) -> Result<Document, FormatError> {
    let input = String::from_utf8_lossy(bytes).to_string();
    let mut lexer = RtfLexer::new(&input);
    let tokens = lexer.tokenize();

    let mut doc = Document::default();
    doc.body.clear();

    let mut blocks = vec![];
    let mut current_para = Paragraph::default();
    let mut current_format = RunFormat::default();
    let mut group_depth = 0;
    let mut in_info = false;

    for token in &tokens {
        match token {
            RtfToken::GroupStart => {
                group_depth += 1;
            }
            RtfToken::GroupEnd => {
                group_depth -= 1;
                if group_depth == 0 { in_info = false; }
            }
            RtfToken::ControlWord(word, param) => {
                match word.as_str() {
                    "par" => {
                        blocks.push(Block::Paragraph(current_para));
                        current_para = Paragraph::default();
                    }
                    "b" => {
                        current_format.bold = param.map(|p| p != 0).unwrap_or(true);
                    }
                    "i" => {
                        current_format.italic = param.map(|p| p != 0).unwrap_or(true);
                    }
                    "ul" => {
                        current_format.underline = param.map(|p| p != 0).unwrap_or(true);
                    }
                    "strike" => {
                        current_format.strikethrough = param.map(|p| p != 0).unwrap_or(true);
                    }
                    "info" => { in_info = true; }
                    _ => {}
                }
            }
            RtfToken::Text(text) if !in_info && group_depth > 0 => {
                if !text.is_empty() {
                    let mut run = Run::new(text.clone());
                    run.format = current_format.clone();
                    current_para.runs.push(run);
                }
            }
            _ => {}
        }
    }

    // Push last paragraph
    if !current_para.runs.is_empty() {
        blocks.push(Block::Paragraph(current_para));
    }

    doc.body = if blocks.is_empty() {
        vec![Block::Paragraph(Paragraph::default())]
    } else {
        blocks
    };

    Ok(doc)
}
