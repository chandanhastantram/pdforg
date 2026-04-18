//! Formula evaluator — evaluates Expr ASTs against a sheet context.

use super::parser::{Expr, BinOpKind, UnaryOpKind};
use super::functions::FUNCTIONS;
use thiserror::Error;
use crate::core::{CellAddress, CellValue, CellError, Sheet, Workbook};
use std::collections::HashSet;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Unknown function: {0}")]
    UnknownFunc(String),
    #[error("Wrong number of arguments for {0}: expected {1}, got {2}")]
    ArgCount(String, String, usize),
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Division by zero")]
    DivZero,
    #[error("Circular reference detected")]
    CircularRef,
    #[error("Invalid reference: {0}")]
    InvalidRef(String),
    #[error("Formula error: {0}")]
    Formula(String),
}

/// Context for formula evaluation
pub struct EvalContext<'a> {
    pub workbook: &'a Workbook,
    pub current_sheet: usize,
    pub current_cell: CellAddress,
    pub call_stack: HashSet<CellAddress>,  // for circular ref detection
}

impl<'a> EvalContext<'a> {
    pub fn new(workbook: &'a Workbook, sheet: usize, cell: CellAddress) -> Self {
        EvalContext {
            workbook,
            current_sheet: sheet,
            current_cell: cell,
            call_stack: HashSet::new(),
        }
    }

    pub fn sheet(&self) -> Option<&Sheet> {
        self.workbook.sheets.get(self.current_sheet)
    }

    pub fn get_cell_value(&self, addr: &CellAddress) -> CellValue {
        let sheet_idx = if let Some(ref name) = addr.sheet {
            self.workbook.sheets.iter().position(|s| &s.name == name)
                .unwrap_or(self.current_sheet)
        } else {
            self.current_sheet
        };

        self.workbook.sheets.get(sheet_idx)
            .and_then(|s| s.get_cell(addr.row, addr.col))
            .map(|c| c.value.clone())
            .unwrap_or(CellValue::Empty)
    }

    pub fn get_range_values(&self, start: &CellAddress, end: &CellAddress) -> Vec<Vec<CellValue>> {
        let sheet_idx = self.current_sheet;
        let sheet = match self.workbook.sheets.get(sheet_idx) {
            Some(s) => s,
            None => return vec![],
        };

        let r1 = start.row.min(end.row);
        let r2 = start.row.max(end.row);
        let c1 = start.col.min(end.col);
        let c2 = start.col.max(end.col);

        (r1..=r2).map(|row| {
            (c1..=c2).map(|col| {
                sheet.get_cell(row, col)
                    .map(|c| c.value.clone())
                    .unwrap_or(CellValue::Empty)
            }).collect()
        }).collect()
    }
}

/// Evaluate an expression in a given context
pub fn eval(expr: &Expr, ctx: &EvalContext) -> Result<CellValue, EvalError> {
    match expr {
        Expr::Number(n) => Ok(CellValue::Number(*n)),
        Expr::Text(s) => Ok(CellValue::Text(s.clone())),
        Expr::Bool(b) => Ok(CellValue::Bool(*b)),
        Expr::Error(e) => Ok(CellValue::Error(parse_error(e))),

        Expr::Percent(inner) => {
            let v = eval(inner, ctx)?;
            match v.as_number() {
                Some(n) => Ok(CellValue::Number(n / 100.0)),
                None => Err(EvalError::TypeError("Expected number for %".into())),
            }
        }

        Expr::CellRef(addr) => Ok(ctx.get_cell_value(addr)),

        Expr::RangeRef(start, end) => {
            // When a range is used as a scalar, return the top-left cell
            Ok(ctx.get_cell_value(start))
        }

        Expr::NamedRange(name) => {
            // Look up named range
            if let Some(range) = ctx.workbook.named_ranges.get(name.as_str()) {
                Ok(ctx.get_cell_value(&range.start))
            } else {
                Ok(CellValue::Error(CellError::Name))
            }
        }

        Expr::Array(rows) => {
            // For now, return first element of array
            if let Some(first_row) = rows.first() {
                if let Some(first_cell) = first_row.first() {
                    return eval(first_cell, ctx);
                }
            }
            Ok(CellValue::Empty)
        }

        Expr::UnaryOp { op, expr } => {
            let v = eval(expr, ctx)?;
            match op {
                UnaryOpKind::Neg => match v.as_number() {
                    Some(n) => Ok(CellValue::Number(-n)),
                    None => Err(EvalError::TypeError("Expected number for negation".into())),
                },
                UnaryOpKind::Plus => Ok(v),
            }
        }

        Expr::BinOp { left, op, right } => eval_binop(left, op, right, ctx),

        Expr::Call { name, args } => eval_call(name, args, ctx),
    }
}

fn eval_binop(left: &Expr, op: &BinOpKind, right: &Expr, ctx: &EvalContext) -> Result<CellValue, EvalError> {
    // Special case for range expansion in aggregate functions (handled in functions)
    let lv = eval(left, ctx)?;
    let rv = eval(right, ctx)?;

    match op {
        BinOpKind::Add => num_op(lv, rv, |a, b| a + b),
        BinOpKind::Sub => num_op(lv, rv, |a, b| a - b),
        BinOpKind::Mul => num_op(lv, rv, |a, b| a * b),
        BinOpKind::Div => {
            let b = rv.as_number().ok_or_else(|| EvalError::TypeError("Expected number".into()))?;
            if b == 0.0 { return Ok(CellValue::Error(CellError::Div0)); }
            num_op(lv, CellValue::Number(b), |a, _| a / b)
        }
        BinOpKind::Pow => num_op(lv, rv, |a, b| a.powf(b)),
        BinOpKind::Concat => {
            let s = format!("{}{}", lv.as_text(), rv.as_text());
            Ok(CellValue::Text(s))
        }
        BinOpKind::Eq => Ok(CellValue::Bool(cell_eq(&lv, &rv))),
        BinOpKind::Ne => Ok(CellValue::Bool(!cell_eq(&lv, &rv))),
        BinOpKind::Lt => Ok(CellValue::Bool(cell_lt(&lv, &rv))),
        BinOpKind::Le => Ok(CellValue::Bool(cell_lt(&lv, &rv) || cell_eq(&lv, &rv))),
        BinOpKind::Gt => Ok(CellValue::Bool(!cell_lt(&lv, &rv) && !cell_eq(&lv, &rv))),
        BinOpKind::Ge => Ok(CellValue::Bool(!cell_lt(&lv, &rv))),
    }
}

fn num_op(a: CellValue, b: CellValue, f: impl Fn(f64, f64) -> f64) -> Result<CellValue, EvalError> {
    let an = a.as_number().ok_or_else(|| EvalError::TypeError(format!("Expected number, got {:?}", a)))?;
    let bn = b.as_number().ok_or_else(|| EvalError::TypeError(format!("Expected number, got {:?}", b)))?;
    Ok(CellValue::Number(f(an, bn)))
}

fn cell_eq(a: &CellValue, b: &CellValue) -> bool {
    match (a, b) {
        (CellValue::Number(x), CellValue::Number(y)) => x == y,
        (CellValue::Text(x), CellValue::Text(y)) => x.to_uppercase() == y.to_uppercase(),
        (CellValue::Bool(x), CellValue::Bool(y)) => x == y,
        (CellValue::Empty, CellValue::Empty) => true,
        _ => false,
    }
}

fn cell_lt(a: &CellValue, b: &CellValue) -> bool {
    match (a, b) {
        (CellValue::Number(x), CellValue::Number(y)) => x < y,
        (CellValue::Text(x), CellValue::Text(y)) => x.to_uppercase() < y.to_uppercase(),
        (CellValue::Bool(x), CellValue::Bool(y)) => !x && *y,
        _ => false,
    }
}

fn eval_call(name: &str, args: &[Expr], ctx: &EvalContext) -> Result<CellValue, EvalError> {
    // Collect range values for functions that need them specially
    let fn_args = collect_fn_args(name, args, ctx)?;

    // Look up and call function
    if let Some(func) = FUNCTIONS.get(name.to_uppercase().as_str()) {
        func(&fn_args).map_err(|e| EvalError::Formula(e))
    } else {
        Err(EvalError::UnknownFunc(name.to_string()))
    }
}

/// Collect function arguments, expanding ranges when needed
fn collect_fn_args(name: &str, args: &[Expr], ctx: &EvalContext) -> Result<Vec<CellValue>, EvalError> {
    // Functions that expand ranges into flat lists of values
    let range_expanding = [
        "SUM", "AVERAGE", "MIN", "MAX", "COUNT", "COUNTA", "COUNTBLANK",
        "PRODUCT", "STDEV", "STDEVP", "VAR", "VARP", "MEDIAN", "MODE",
    ];
    let expands = range_expanding.contains(&name.to_uppercase().as_str());

    let mut result = vec![];
    for arg in args {
        match arg {
            Expr::RangeRef(start, end) if expands => {
                let values = ctx.get_range_values(start, end);
                for row in values {
                    for v in row {
                        result.push(v);
                    }
                }
            }
            _ => result.push(eval(arg, ctx)?),
        }
    }
    Ok(result)
}

fn parse_error(s: &str) -> CellError {
    match s {
        "#DIV/0!" => CellError::Div0,
        "#N/A" => CellError::NA,
        "#NAME?" => CellError::Name,
        "#NULL!" => CellError::Null,
        "#NUM!" => CellError::Num,
        "#REF!" => CellError::Ref,
        "#VALUE!" => CellError::Value,
        _ => CellError::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::parser::parse_formula;

    fn eval_str(formula: &str, wb: &Workbook) -> CellValue {
        let expr = parse_formula(formula).unwrap();
        let ctx = EvalContext::new(wb, 0, CellAddress::new(0, 0));
        eval(&expr, &ctx).unwrap_or(CellValue::Error(CellError::Value))
    }

    #[test]
    fn test_eval_arithmetic() {
        let wb = Workbook::default();
        assert_eq!(eval_str("=2+3", &wb), CellValue::Number(5.0));
        assert_eq!(eval_str("=10/4", &wb), CellValue::Number(2.5));
        assert_eq!(eval_str("=2^10", &wb), CellValue::Number(1024.0));
    }

    #[test]
    fn test_eval_sum() {
        let mut wb = Workbook::default();
        let sheet = wb.sheets.get_mut(0).unwrap();
        use crate::core::Cell;
        sheet.set_cell(0, 0, Cell { value: CellValue::Number(1.0), ..Default::default() });
        sheet.set_cell(1, 0, Cell { value: CellValue::Number(2.0), ..Default::default() });
        sheet.set_cell(2, 0, Cell { value: CellValue::Number(3.0), ..Default::default() });
        assert_eq!(eval_str("=SUM(A1:A3)", &wb), CellValue::Number(6.0));
    }

    #[test]
    fn test_eval_if() {
        let wb = Workbook::default();
        assert_eq!(eval_str("=IF(1>0,\"yes\",\"no\")", &wb), CellValue::Text("yes".into()));
        assert_eq!(eval_str("=IF(0>1,\"yes\",\"no\")", &wb), CellValue::Text("no".into()));
    }

    #[test]
    fn test_eval_concatenate() {
        let wb = Workbook::default();
        assert_eq!(eval_str("=\"Hello\"&\" \"&\"World\"", &wb), CellValue::Text("Hello World".into()));
    }
}
