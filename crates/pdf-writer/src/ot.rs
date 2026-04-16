//! Operational Transform (OT) engine for collaborative text editing.
//!
//! Based on the OT algorithm originally described by Ellis & Gibbs (1989),
//! with extensions for rich text attributes. Compatible with the Quill Delta format.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use pdf_core::{Document, Paragraph, Run, Block};

/// Attribute map for styled text
pub type Attrs = HashMap<String, serde_json::Value>;

/// A single operation in the OT model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Op {
    /// Retain N characters (optionally applying attributes)
    Retain { n: usize, attrs: Option<Attrs> },
    /// Insert text with optional attributes
    Insert { text: String, attrs: Option<Attrs> },
    /// Delete N characters
    Delete { n: usize },
}

impl Op {
    pub fn retain(n: usize) -> Self { Op::Retain { n, attrs: None } }
    pub fn retain_attrs(n: usize, attrs: Attrs) -> Self { Op::Retain { n, attrs: Some(attrs) } }
    pub fn insert(text: impl Into<String>) -> Self { Op::Insert { text: text.into(), attrs: None } }
    pub fn insert_attrs(text: impl Into<String>, attrs: Attrs) -> Self {
        Op::Insert { text: text.into(), attrs: Some(attrs) }
    }
    pub fn delete(n: usize) -> Self { Op::Delete { n } }

    /// Length of chars this op consumes from the input document
    pub fn consumes(&self) -> usize {
        match self {
            Op::Retain { n, .. } => *n,
            Op::Insert { .. } => 0,
            Op::Delete { n } => *n,
        }
    }

    /// Length of chars this op produces in the output document
    pub fn produces(&self) -> usize {
        match self {
            Op::Retain { n, .. } => *n,
            Op::Insert { text, .. } => text.chars().count(),
            Op::Delete { .. } => 0,
        }
    }
}

/// Priority for transform — determines who "wins" on concurrent inserts at the same position
#[derive(Debug, Clone, PartialEq)]
pub enum Priority { Left, Right }

/// Compose two operation sequences into a single equivalent sequence.
///
/// compose(a, b) produces a single op sequence that has the same effect as
/// applying `a` first and then `b`.
pub fn compose(a: &[Op], b: &[Op]) -> Vec<Op> {
    let mut result = vec![];
    let mut a_iter = a.iter().cloned().peekable();
    let mut b_iter = b.iter().cloned().peekable();

    let mut cur_a: Option<Op> = a_iter.next();
    let mut cur_b: Option<Op> = b_iter.next();

    loop {
        match (cur_a.take(), cur_b.take()) {
            (None, None) => break,
            (Some(op), None) => { push_op(&mut result, op); cur_a = a_iter.next(); }
            (None, Some(op)) => { push_op(&mut result, op); cur_b = b_iter.next(); }

            // b inserts — always come first
            (a_op, Some(Op::Insert { text, attrs })) => {
                push_op(&mut result, Op::Insert { text, attrs });
                cur_a = a_op;
                cur_b = b_iter.next();
            }

            // a deletes — pass through
            (Some(Op::Delete { n }), b_op) => {
                push_op(&mut result, Op::Delete { n });
                cur_a = a_iter.next();
                cur_b = b_op;
            }

            (Some(Op::Insert { text: a_text, .. }), Some(Op::Delete { n })) => {
                let a_len = a_text.chars().count();
                match a_len.cmp(&n) {
                    std::cmp::Ordering::Equal => {
                        cur_a = a_iter.next();
                        cur_b = b_iter.next();
                    }
                    std::cmp::Ordering::Less => {
                        cur_a = a_iter.next();
                        cur_b = Some(Op::Delete { n: n - a_len });
                    }
                    std::cmp::Ordering::Greater => {
                        let remaining: String = a_text.chars().skip(n).collect();
                        cur_a = Some(Op::Insert { text: remaining, attrs: None });
                        cur_b = b_iter.next();
                    }
                }
            }

            (Some(Op::Insert { text, attrs }), Some(Op::Retain { n, attrs: b_attrs })) => {
                let a_len = text.chars().count();
                match a_len.cmp(&n) {
                    std::cmp::Ordering::Equal => {
                        let merged = merge_attrs(attrs, b_attrs);
                        push_op(&mut result, Op::Insert { text, attrs: merged });
                        cur_a = a_iter.next();
                        cur_b = b_iter.next();
                    }
                    std::cmp::Ordering::Less => {
                        let merged = merge_attrs(attrs, b_attrs.clone());
                        push_op(&mut result, Op::Insert { text, attrs: merged });
                        cur_a = a_iter.next();
                        cur_b = Some(Op::Retain { n: n - a_len, attrs: b_attrs });
                    }
                    std::cmp::Ordering::Greater => {
                        let (head, tail) = split_text(&text, n);
                        let merged = merge_attrs(Some(head.clone()).map(|_| attrs.clone()).flatten(), b_attrs);
                        push_op(&mut result, Op::Insert { text: head, attrs: merged });
                        cur_a = Some(Op::Insert { text: tail, attrs });
                        cur_b = b_iter.next();
                    }
                }
            }

            (Some(Op::Retain { n: a_n, attrs: a_attrs }), Some(Op::Retain { n: b_n, attrs: b_attrs })) => {
                let min = a_n.min(b_n);
                let merged = merge_attrs(a_attrs.clone(), b_attrs.clone());
                push_op(&mut result, Op::Retain { n: min, attrs: merged });
                if a_n > min { cur_a = Some(Op::Retain { n: a_n - min, attrs: a_attrs }); } else { cur_a = a_iter.next(); }
                if b_n > min { cur_b = Some(Op::Retain { n: b_n - min, attrs: b_attrs }); } else { cur_b = b_iter.next(); }
            }

            (Some(Op::Retain { n: a_n, attrs: a_attrs }), Some(Op::Delete { n: b_n })) => {
                let min = a_n.min(b_n);
                push_op(&mut result, Op::Delete { n: min });
                if a_n > min { cur_a = Some(Op::Retain { n: a_n - min, attrs: a_attrs }); } else { cur_a = a_iter.next(); }
                if b_n > min { cur_b = Some(Op::Delete { n: b_n - min }); } else { cur_b = b_iter.next(); }
            }
        }
    }

    result
}

/// Transform two concurrent op sequences so they can be applied after each other.
///
/// Returns (a', b') where:
/// - a' is a transformed to apply after b
/// - b' is b transformed to apply after a
pub fn transform(a: &[Op], b: &[Op], priority: Priority) -> (Vec<Op>, Vec<Op>) {
    let mut a_prime = vec![];
    let mut b_prime = vec![];

    let mut a_iter = a.iter().cloned().peekable();
    let mut b_iter = b.iter().cloned().peekable();

    let mut cur_a = a_iter.next();
    let mut cur_b = b_iter.next();

    loop {
        match (cur_a.take(), cur_b.take()) {
            (None, None) => break,
            (Some(op), None) => { push_op(&mut a_prime, op); cur_a = a_iter.next(); }
            (None, Some(op)) => { push_op(&mut b_prime, op); cur_b = b_iter.next(); }

            (Some(Op::Insert { text: a_text, attrs: a_attrs }), b_op) => {
                let len = a_text.chars().count();
                if priority == Priority::Left || matches!(b_op, Some(Op::Insert { .. })) {
                    push_op(&mut a_prime, Op::Insert { text: a_text, attrs: a_attrs });
                    push_op(&mut b_prime, Op::Retain { n: len, attrs: None });
                    cur_b = b_op;
                } else {
                    if let Some(Op::Insert { text: b_text, attrs: b_attrs }) = b_op {
                        let b_len = b_text.chars().count();
                        push_op(&mut b_prime, Op::Insert { text: b_text, attrs: b_attrs });
                        push_op(&mut a_prime, Op::Retain { n: b_len, attrs: None });
                        cur_a = Some(Op::Insert { text: a_text, attrs: a_attrs });
                    } else {
                        push_op(&mut a_prime, Op::Insert { text: a_text, attrs: a_attrs });
                        push_op(&mut b_prime, Op::Retain { n: len, attrs: None });
                        cur_b = b_op;
                    }
                }
                cur_a = a_iter.next();
            }

            (a_op, Some(Op::Insert { text: b_text, attrs: b_attrs })) => {
                let len = b_text.chars().count();
                push_op(&mut b_prime, Op::Insert { text: b_text, attrs: b_attrs });
                push_op(&mut a_prime, Op::Retain { n: len, attrs: None });
                cur_a = a_op;
                cur_b = b_iter.next();
            }

            (Some(Op::Delete { n: a_n }), Some(Op::Delete { n: b_n })) => {
                let min = a_n.min(b_n);
                if a_n > min { cur_a = Some(Op::Delete { n: a_n - min }); } else { cur_a = a_iter.next(); }
                if b_n > min { cur_b = Some(Op::Delete { n: b_n - min }); } else { cur_b = b_iter.next(); }
            }

            (Some(Op::Delete { n: a_n }), Some(Op::Retain { n: b_n, .. })) => {
                let min = a_n.min(b_n);
                push_op(&mut a_prime, Op::Delete { n: min });
                if a_n > min { cur_a = Some(Op::Delete { n: a_n - min }); } else { cur_a = a_iter.next(); }
                if b_n > min { cur_b = Some(Op::Retain { n: b_n - min, attrs: None }); } else { cur_b = b_iter.next(); }
            }

            (Some(Op::Retain { n: a_n, attrs: a_attrs }), Some(Op::Delete { n: b_n })) => {
                let min = a_n.min(b_n);
                push_op(&mut b_prime, Op::Delete { n: min });
                if a_n > min { cur_a = Some(Op::Retain { n: a_n - min, attrs: a_attrs }); } else { cur_a = a_iter.next(); }
                if b_n > min { cur_b = Some(Op::Delete { n: b_n - min }); } else { cur_b = b_iter.next(); }
            }

            (Some(Op::Retain { n: a_n, attrs: a_attrs }), Some(Op::Retain { n: b_n, attrs: b_attrs })) => {
                let min = a_n.min(b_n);
                push_op(&mut a_prime, Op::Retain { n: min, attrs: a_attrs.clone() });
                push_op(&mut b_prime, Op::Retain { n: min, attrs: b_attrs.clone() });
                if a_n > min { cur_a = Some(Op::Retain { n: a_n - min, attrs: a_attrs }); } else { cur_a = a_iter.next(); }
                if b_n > min { cur_b = Some(Op::Retain { n: b_n - min, attrs: b_attrs }); } else { cur_b = b_iter.next(); }
            }
        }
    }

    (a_prime, b_prime)
}

/// Apply ops to a plain-text string (simplified for text content)
pub fn apply_to_text(text: &str, ops: &[Op]) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::new();
    let mut pos = 0;

    for op in ops {
        match op {
            Op::Retain { n, .. } => {
                let end = (pos + n).min(chars.len());
                result.extend(&chars[pos..end]);
                pos = end;
            }
            Op::Insert { text: ins, .. } => {
                result.push_str(ins);
            }
            Op::Delete { n } => {
                pos = (pos + n).min(chars.len());
            }
        }
    }

    // Append any remaining
    if pos < chars.len() {
        result.extend(&chars[pos..]);
    }

    result
}

/// Compute the inverse of an op sequence (for undo)
pub fn invert(ops: &[Op], original_text: &str) -> Vec<Op> {
    let chars: Vec<char> = original_text.chars().collect();
    let mut result = vec![];
    let mut pos = 0;

    for op in ops {
        match op {
            Op::Retain { n, .. } => {
                push_op(&mut result, Op::Retain { n: *n, attrs: None });
                pos += n;
            }
            Op::Insert { text, .. } => {
                push_op(&mut result, Op::Delete { n: text.chars().count() });
            }
            Op::Delete { n } => {
                let text: String = chars[pos..((pos + n).min(chars.len()))].iter().collect();
                push_op(&mut result, Op::Insert { text, attrs: None });
                pos += n;
            }
        }
    }

    result
}

// ---- helpers ----

fn push_op(ops: &mut Vec<Op>, op: Op) {
    match (ops.last_mut(), &op) {
        (Some(Op::Retain { n: last_n, .. }), Op::Retain { n, .. }) => { *last_n += n; return; }
        (Some(Op::Delete { n: last_n }), Op::Delete { n }) => { *last_n += n; return; }
        (Some(Op::Insert { text: last_t, .. }), Op::Insert { text, .. }) => {
            last_t.push_str(text); return;
        }
        _ => {}
    }
    ops.push(op);
}

fn merge_attrs(a: Option<Attrs>, b: Option<Attrs>) -> Option<Attrs> {
    match (a, b) {
        (None, None) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (Some(mut a), Some(b)) => { a.extend(b); Some(a) }
    }
}

fn split_text(text: &str, n: usize) -> (String, String) {
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(n).collect();
    let tail: String = chars.collect();
    (head, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_to_text_insert() {
        let ops = vec![Op::insert("Hello")];
        assert_eq!(apply_to_text("", &ops), "Hello");
    }

    #[test]
    fn test_apply_to_text_delete() {
        let ops = vec![Op::retain(5), Op::delete(6)];
        assert_eq!(apply_to_text("Hello World", &ops), "Hello");
    }

    #[test]
    fn test_compose_basic() {
        let a = vec![Op::insert("Hello")];
        let b = vec![Op::retain(5), Op::insert(" World")];
        let composed = compose(&a, &b);
        assert_eq!(apply_to_text("", &composed), "Hello World");
    }

    #[test]
    fn test_transform_concurrent() {
        let a = vec![Op::insert("X")];
        let b = vec![Op::insert("Y")];
        let (a_prime, b_prime) = transform(&a, &b, Priority::Left);
        let ab = compose(&a, &b_prime);
        let ba = compose(&b, &a_prime);
        // Both should produce the same result
        assert_eq!(apply_to_text("", &ab), apply_to_text("", &ba));
    }

    #[test]
    fn test_invert() {
        let original = "Hello World";
        let ops = vec![Op::retain(5), Op::delete(6)];
        let inv = invert(&ops, original);
        let after_delete = apply_to_text(original, &ops);
        let restored = apply_to_text(&after_delete, &inv);
        assert_eq!(restored, original);
    }
}
