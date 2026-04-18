//! Built-in spreadsheet functions (300+ planned, 80+ implemented here).
//!
//! All functions take `&[CellValue]` and return `Result<CellValue, String>`.

use crate::core::{CellValue, CellError};
use std::collections::HashMap;

pub type FnImpl = fn(&[CellValue]) -> Result<CellValue, String>;

/// Global function registry
pub static FUNCTIONS: std::sync::LazyLock<HashMap<&'static str, FnImpl>> =
    std::sync::LazyLock::new(|| {
        let mut m: HashMap<&'static str, FnImpl> = HashMap::new();

        // ── Math & Trig ──────────────────────────────────────────────────────
        m.insert("SUM",      fn_sum);
        m.insert("SUMIF",    fn_sumif);
        m.insert("SUMIFS",   fn_sumifs);
        m.insert("PRODUCT",  fn_product);
        m.insert("ABS",      fn_abs);
        m.insert("ROUND",    fn_round);
        m.insert("ROUNDUP",  fn_roundup);
        m.insert("ROUNDDOWN",fn_rounddown);
        m.insert("CEILING",  fn_ceiling);
        m.insert("FLOOR",    fn_floor);
        m.insert("TRUNC",    fn_trunc);
        m.insert("INT",      fn_int);
        m.insert("MOD",      fn_mod);
        m.insert("POWER",    fn_power);
        m.insert("SQRT",     fn_sqrt);
        m.insert("SQRTPI",   fn_sqrtpi);
        m.insert("EXP",      fn_exp);
        m.insert("LN",       fn_ln);
        m.insert("LOG",      fn_log);
        m.insert("LOG10",    fn_log10);
        m.insert("PI",       fn_pi);
        m.insert("SIN",      fn_sin);
        m.insert("COS",      fn_cos);
        m.insert("TAN",      fn_tan);
        m.insert("ASIN",     fn_asin);
        m.insert("ACOS",     fn_acos);
        m.insert("ATAN",     fn_atan);
        m.insert("ATAN2",    fn_atan2);
        m.insert("DEGREES",  fn_degrees);
        m.insert("RADIANS",  fn_radians);
        m.insert("SIGN",     fn_sign);
        m.insert("RAND",     fn_rand);
        m.insert("RANDBETWEEN", fn_randbetween);
        m.insert("FACT",     fn_fact);
        m.insert("COMBIN",   fn_combin);
        m.insert("PERMUT",   fn_permut);
        m.insert("GCD",      fn_gcd);
        m.insert("LCM",      fn_lcm);
        m.insert("EVEN",     fn_even);
        m.insert("ODD",      fn_odd);
        m.insert("MROUND",   fn_mround);

        // ── Statistical ──────────────────────────────────────────────────────
        m.insert("AVERAGE",  fn_average);
        m.insert("AVERAGEIF",fn_averageif);
        m.insert("MIN",      fn_min);
        m.insert("MAX",      fn_max);
        m.insert("MEDIAN",   fn_median);
        m.insert("MODE",     fn_mode);
        m.insert("COUNT",    fn_count);
        m.insert("COUNTA",   fn_counta);
        m.insert("COUNTBLANK", fn_countblank);
        m.insert("COUNTIF",  fn_countif);
        m.insert("COUNTIFS", fn_countifs);
        m.insert("STDEV",    fn_stdev);
        m.insert("STDEVP",   fn_stdevp);
        m.insert("VAR",      fn_var);
        m.insert("VARP",     fn_varp);
        m.insert("LARGE",    fn_large);
        m.insert("SMALL",    fn_small);
        m.insert("RANK",     fn_rank);
        m.insert("PERCENTILE", fn_percentile);
        m.insert("QUARTILE", fn_quartile);
        m.insert("CORREL",   fn_correl);
        m.insert("COVAR",    fn_covar);
        m.insert("SLOPE",    fn_slope);
        m.insert("INTERCEPT",fn_intercept);

        // ── Text ─────────────────────────────────────────────────────────────
        m.insert("LEN",      fn_len);
        m.insert("LEFT",     fn_left);
        m.insert("RIGHT",    fn_right);
        m.insert("MID",      fn_mid);
        m.insert("UPPER",    fn_upper);
        m.insert("LOWER",    fn_lower);
        m.insert("PROPER",   fn_proper);
        m.insert("TRIM",     fn_trim);
        m.insert("LTRIM",    fn_ltrim);
        m.insert("RTRIM",    fn_rtrim);
        m.insert("CONCATENATE", fn_concatenate);
        m.insert("CONCAT",   fn_concat);
        m.insert("TEXTJOIN", fn_textjoin);
        m.insert("REPLACE",  fn_replace);
        m.insert("SUBSTITUTE",fn_substitute);
        m.insert("FIND",     fn_find);
        m.insert("SEARCH",   fn_search);
        m.insert("TEXT",     fn_text);
        m.insert("VALUE",    fn_value);
        m.insert("FIXED",    fn_fixed);
        m.insert("REPT",     fn_rept);
        m.insert("CODE",     fn_code);
        m.insert("CHAR",     fn_char);
        m.insert("EXACT",    fn_exact);
        m.insert("T",        fn_t);

        // ── Logical ──────────────────────────────────────────────────────────
        m.insert("IF",       fn_if);
        m.insert("IFS",      fn_ifs);
        m.insert("AND",      fn_and);
        m.insert("OR",       fn_or);
        m.insert("NOT",      fn_not);
        m.insert("XOR",      fn_xor);
        m.insert("IFERROR",  fn_iferror);
        m.insert("IFNA",     fn_ifna);
        m.insert("SWITCH",   fn_switch);
        m.insert("TRUE",     fn_true);
        m.insert("FALSE",    fn_false);

        // ── Lookup ───────────────────────────────────────────────────────────
        m.insert("VLOOKUP",  fn_vlookup);
        m.insert("HLOOKUP",  fn_hlookup);
        m.insert("INDEX",    fn_index);
        m.insert("MATCH",    fn_match);
        m.insert("CHOOSE",   fn_choose);
        m.insert("OFFSET",   fn_offset);
        m.insert("ROW",      fn_row);
        m.insert("COLUMN",   fn_column);
        m.insert("ROWS",     fn_rows);
        m.insert("COLUMNS",  fn_columns);
        m.insert("TRANSPOSE",fn_transpose);

        // ── Date & Time ──────────────────────────────────────────────────────
        m.insert("NOW",      fn_now);
        m.insert("TODAY",    fn_today);
        m.insert("DATE",     fn_date);
        m.insert("TIME",     fn_time);
        m.insert("YEAR",     fn_year);
        m.insert("MONTH",    fn_month);
        m.insert("DAY",      fn_day);
        m.insert("HOUR",     fn_hour);
        m.insert("MINUTE",   fn_minute);
        m.insert("SECOND",   fn_second);
        m.insert("WEEKDAY",  fn_weekday);
        m.insert("WEEKNUM",  fn_weeknum);
        m.insert("EDATE",    fn_edate);
        m.insert("EOMONTH",  fn_eomonth);
        m.insert("NETWORKDAYS", fn_networkdays);
        m.insert("WORKDAY",  fn_workday);
        m.insert("DATEDIF",  fn_datedif);
        m.insert("DATEVALUE",fn_datevalue);
        m.insert("TIMEVALUE",fn_timevalue);

        // ── Information ──────────────────────────────────────────────────────
        m.insert("ISNUMBER", fn_isnumber);
        m.insert("ISTEXT",   fn_istext);
        m.insert("ISBLANK",  fn_isblank);
        m.insert("ISERROR",  fn_iserror);
        m.insert("ISERR",    fn_iserr);
        m.insert("ISNA",     fn_isna);
        m.insert("ISLOGICAL",fn_islogical);
        m.insert("ISODD",    fn_isodd);
        m.insert("ISEVEN",   fn_iseven);
        m.insert("NA",       fn_na);
        m.insert("ERROR.TYPE", fn_error_type);
        m.insert("CELL",     fn_cell);
        m.insert("TYPE",     fn_type);
        m.insert("N",        fn_n);

        // ── Financial ────────────────────────────────────────────────────────
        m.insert("PMT",      fn_pmt);
        m.insert("PV",       fn_pv);
        m.insert("FV",       fn_fv);
        m.insert("RATE",     fn_rate);
        m.insert("NPER",     fn_nper);
        m.insert("NPV",      fn_npv);
        m.insert("IRR",      fn_irr);
        m.insert("IPMT",     fn_ipmt);
        m.insert("PPMT",     fn_ppmt);

        m
    });

// ─── Helper macros ───────────────────────────────────────────────────────────

macro_rules! require_num {
    ($v:expr) => {
        $v.as_number().ok_or_else(|| format!("Expected number, got {:?}", $v))?
    };
}

macro_rules! require_text {
    ($v:expr) => {
        match $v {
            CellValue::Text(s) => s.clone(),
            other => other.as_text(),
        }
    };
}

fn nums_from_args(args: &[CellValue]) -> Vec<f64> {
    args.iter().filter_map(|v| v.as_number()).collect()
}

fn to_bool(v: &CellValue) -> bool {
    match v {
        CellValue::Bool(b) => *b,
        CellValue::Number(n) => *n != 0.0,
        CellValue::Text(s) => !s.is_empty(),
        CellValue::Empty => false,
        _ => false,
    }
}

// ─── Math & Trig ─────────────────────────────────────────────────────────────

fn fn_sum(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(nums_from_args(args).iter().sum()))
}

fn fn_sumif(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 2 { return Err("SUMIF requires at least 2 arguments".into()); }
    // Simplified: sum if value matches criteria
    let criteria = &args[1];
    let sum: f64 = args[0..1].iter()
        .zip(args.get(2..).unwrap_or(&args[0..1]))
        .filter_map(|(check, sum_v)| {
            if check == criteria { sum_v.as_number() } else { None }
        })
        .sum();
    Ok(CellValue::Number(sum))
}

fn fn_sumifs(args: &[CellValue]) -> Result<CellValue, String> {
    // Simplified stub
    fn_sum(args)
}

fn fn_product(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(nums_from_args(args).iter().product()))
}

fn fn_abs(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    Ok(CellValue::Number(n.abs()))
}

fn fn_round(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let places = if args.len() > 1 { require_num!(&args[1]) as i32 } else { 0 };
    let factor = 10f64.powi(places);
    Ok(CellValue::Number((n * factor).round() / factor))
}

fn fn_roundup(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let places = if args.len() > 1 { require_num!(&args[1]) as i32 } else { 0 };
    let factor = 10f64.powi(places);
    Ok(CellValue::Number((n * factor).ceil() / factor))
}

fn fn_rounddown(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let places = if args.len() > 1 { require_num!(&args[1]) as i32 } else { 0 };
    let factor = 10f64.powi(places);
    Ok(CellValue::Number((n * factor).floor() / factor))
}

fn fn_ceiling(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let sig = if args.len() > 1 { require_num!(&args[1]) } else { 1.0 };
    if sig == 0.0 { return Ok(CellValue::Number(0.0)); }
    Ok(CellValue::Number((n / sig).ceil() * sig))
}

fn fn_floor(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let sig = if args.len() > 1 { require_num!(&args[1]) } else { 1.0 };
    if sig == 0.0 { return Ok(CellValue::Number(0.0)); }
    Ok(CellValue::Number((n / sig).floor() * sig))
}

fn fn_trunc(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    Ok(CellValue::Number(n.trunc()))
}

fn fn_int(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    Ok(CellValue::Number(n.floor()))
}

fn fn_mod(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let d = require_num!(&args[1]);
    if d == 0.0 { return Ok(CellValue::Error(CellError::Div0)); }
    Ok(CellValue::Number(n - d * (n / d).floor()))
}

fn fn_power(args: &[CellValue]) -> Result<CellValue, String> {
    let base = require_num!(&args[0]);
    let exp = require_num!(&args[1]);
    Ok(CellValue::Number(base.powf(exp)))
}

fn fn_sqrt(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    if n < 0.0 { return Ok(CellValue::Error(CellError::Num)); }
    Ok(CellValue::Number(n.sqrt()))
}

fn fn_sqrtpi(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    Ok(CellValue::Number((n * std::f64::consts::PI).sqrt()))
}

fn fn_exp(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).exp()))
}

fn fn_ln(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    if n <= 0.0 { return Ok(CellValue::Error(CellError::Num)); }
    Ok(CellValue::Number(n.ln()))
}

fn fn_log(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let base = if args.len() > 1 { require_num!(&args[1]) } else { 10.0 };
    if n <= 0.0 || base <= 0.0 || base == 1.0 { return Ok(CellValue::Error(CellError::Num)); }
    Ok(CellValue::Number(n.log(base)))
}

fn fn_log10(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    if n <= 0.0 { return Ok(CellValue::Error(CellError::Num)); }
    Ok(CellValue::Number(n.log10()))
}

fn fn_pi(_: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(std::f64::consts::PI))
}

fn fn_sin(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).sin()))
}
fn fn_cos(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).cos()))
}
fn fn_tan(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).tan()))
}
fn fn_asin(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).asin()))
}
fn fn_acos(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).acos()))
}
fn fn_atan(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).atan()))
}
fn fn_atan2(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).atan2(require_num!(&args[1]))))
}
fn fn_degrees(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).to_degrees()))
}
fn fn_radians(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_num!(&args[0]).to_radians()))
}
fn fn_sign(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    Ok(CellValue::Number(if n > 0.0 { 1.0 } else if n < 0.0 { -1.0 } else { 0.0 }))
}
fn fn_rand(_: &[CellValue]) -> Result<CellValue, String> {
    // Without a proper RNG dependency, use a hash-based pseudo-random
    use std::time::{SystemTime, UNIX_EPOCH};
    let ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    Ok(CellValue::Number((ns as f64) / (u32::MAX as f64)))
}
fn fn_randbetween(args: &[CellValue]) -> Result<CellValue, String> {
    let lo = require_num!(&args[0]);
    let hi = require_num!(&args[1]);
    let r = match fn_rand(&[])? { CellValue::Number(n) => n, _ => 0.5 };
    Ok(CellValue::Number((lo + r * (hi - lo)).floor()))
}
fn fn_fact(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]) as u64;
    Ok(CellValue::Number((1..=n).product::<u64>() as f64))
}
fn fn_combin(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]) as u64;
    let k = require_num!(&args[1]) as u64;
    if k > n { return Ok(CellValue::Number(0.0)); }
    let num: u64 = ((n-k+1)..=n).product();
    let den: u64 = (1..=k).product();
    Ok(CellValue::Number((num / den) as f64))
}
fn fn_permut(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]) as u64;
    let k = require_num!(&args[1]) as u64;
    Ok(CellValue::Number(((n-k+1)..=n).product::<u64>() as f64))
}
fn fn_gcd(args: &[CellValue]) -> Result<CellValue, String> {
    fn gcd(a: u64, b: u64) -> u64 { if b == 0 { a } else { gcd(b, a % b) } }
    let ns: Vec<u64> = args.iter().filter_map(|v| v.as_number().map(|n| n as u64)).collect();
    Ok(CellValue::Number(ns.iter().copied().reduce(gcd).unwrap_or(0) as f64))
}
fn fn_lcm(args: &[CellValue]) -> Result<CellValue, String> {
    fn gcd(a: u64, b: u64) -> u64 { if b == 0 { a } else { gcd(b, a % b) } }
    fn lcm(a: u64, b: u64) -> u64 { a / gcd(a, b) * b }
    let ns: Vec<u64> = args.iter().filter_map(|v| v.as_number().map(|n| n as u64)).collect();
    Ok(CellValue::Number(ns.iter().copied().reduce(lcm).unwrap_or(0) as f64))
}
fn fn_even(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let r = if n >= 0.0 { (n / 2.0).ceil() * 2.0 } else { (n / 2.0).floor() * 2.0 };
    Ok(CellValue::Number(r))
}
fn fn_odd(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let r = if n >= 0.0 { (n / 2.0).ceil() * 2.0 + 1.0 } else { (n / 2.0).floor() * 2.0 - 1.0 };
    Ok(CellValue::Number(r))
}
fn fn_mround(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let m = require_num!(&args[1]);
    if m == 0.0 { return Ok(CellValue::Number(0.0)); }
    Ok(CellValue::Number((n / m).round() * m))
}

// ─── Statistical ─────────────────────────────────────────────────────────────

fn fn_average(args: &[CellValue]) -> Result<CellValue, String> {
    let ns = nums_from_args(args);
    if ns.is_empty() { return Ok(CellValue::Error(CellError::Div0)); }
    Ok(CellValue::Number(ns.iter().sum::<f64>() / ns.len() as f64))
}

fn fn_averageif(args: &[CellValue]) -> Result<CellValue, String> {
    fn_average(args) // simplified
}

fn fn_min(args: &[CellValue]) -> Result<CellValue, String> {
    let ns = nums_from_args(args);
    Ok(CellValue::Number(ns.iter().copied().fold(f64::INFINITY, f64::min)))
}

fn fn_max(args: &[CellValue]) -> Result<CellValue, String> {
    let ns = nums_from_args(args);
    Ok(CellValue::Number(ns.iter().copied().fold(f64::NEG_INFINITY, f64::max)))
}

fn fn_median(args: &[CellValue]) -> Result<CellValue, String> {
    let mut ns = nums_from_args(args);
    if ns.is_empty() { return Ok(CellValue::Error(CellError::NA)); }
    ns.sort_by(f64::total_cmp);
    let mid = ns.len() / 2;
    let m = if ns.len() % 2 == 0 { (ns[mid - 1] + ns[mid]) / 2.0 } else { ns[mid] };
    Ok(CellValue::Number(m))
}

fn fn_mode(args: &[CellValue]) -> Result<CellValue, String> {
    let ns = nums_from_args(args);
    let mut freq: HashMap<u64, (f64, usize)> = HashMap::new();
    for n in &ns {
        let key = n.to_bits();
        let entry = freq.entry(key).or_insert((*n, 0));
        entry.1 += 1;
    }
    freq.into_values().max_by_key(|(_, c)| *c).map(|(v, _)| CellValue::Number(v))
        .ok_or_else(|| "No data".into())
}

fn fn_count(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(args.iter().filter(|v| v.is_number()).count() as f64))
}

fn fn_counta(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(args.iter().filter(|v| !v.is_empty()).count() as f64))
}

fn fn_countblank(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(args.iter().filter(|v| v.is_empty()).count() as f64))
}

fn fn_countif(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 2 { return Err("COUNTIF requires 2 args".into()); }
    let criteria = &args[1];
    let count = args[..args.len()-1].iter().filter(|v| *v == criteria).count();
    Ok(CellValue::Number(count as f64))
}

fn fn_countifs(args: &[CellValue]) -> Result<CellValue, String> {
    fn_countif(args)
}

fn fn_stdev(args: &[CellValue]) -> Result<CellValue, String> {
    let ns = nums_from_args(args);
    if ns.len() < 2 { return Ok(CellValue::Error(CellError::Div0)); }
    let mean = ns.iter().sum::<f64>() / ns.len() as f64;
    let var = ns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (ns.len() - 1) as f64;
    Ok(CellValue::Number(var.sqrt()))
}

fn fn_stdevp(args: &[CellValue]) -> Result<CellValue, String> {
    let ns = nums_from_args(args);
    if ns.is_empty() { return Ok(CellValue::Error(CellError::Div0)); }
    let mean = ns.iter().sum::<f64>() / ns.len() as f64;
    let var = ns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / ns.len() as f64;
    Ok(CellValue::Number(var.sqrt()))
}

fn fn_var(args: &[CellValue]) -> Result<CellValue, String> {
    let ns = nums_from_args(args);
    if ns.len() < 2 { return Ok(CellValue::Error(CellError::Div0)); }
    let mean = ns.iter().sum::<f64>() / ns.len() as f64;
    Ok(CellValue::Number(ns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (ns.len() - 1) as f64))
}

fn fn_varp(args: &[CellValue]) -> Result<CellValue, String> {
    let ns = nums_from_args(args);
    if ns.is_empty() { return Ok(CellValue::Error(CellError::Div0)); }
    let mean = ns.iter().sum::<f64>() / ns.len() as f64;
    Ok(CellValue::Number(ns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / ns.len() as f64))
}

fn fn_large(args: &[CellValue]) -> Result<CellValue, String> {
    let mut ns = nums_from_args(&args[..args.len()-1]);
    let k = require_num!(args.last().unwrap()) as usize;
    ns.sort_by(|a, b| b.total_cmp(a));
    ns.get(k - 1).map(|n| CellValue::Number(*n)).ok_or_else(|| "Index out of range".into())
}

fn fn_small(args: &[CellValue]) -> Result<CellValue, String> {
    let mut ns = nums_from_args(&args[..args.len()-1]);
    let k = require_num!(args.last().unwrap()) as usize;
    ns.sort_by(f64::total_cmp);
    ns.get(k - 1).map(|n| CellValue::Number(*n)).ok_or_else(|| "Index out of range".into())
}

fn fn_rank(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let ns = nums_from_args(&args[1..]);
    let rank = ns.iter().filter(|&&x| x > n).count() + 1;
    Ok(CellValue::Number(rank as f64))
}

fn fn_percentile(args: &[CellValue]) -> Result<CellValue, String> {
    let mut ns = nums_from_args(&args[..args.len()-1]);
    let k = require_num!(args.last().unwrap());
    ns.sort_by(f64::total_cmp);
    let idx = k * (ns.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;
    let val = ns[lo] + frac * (ns.get(hi).copied().unwrap_or(ns[lo]) - ns[lo]);
    Ok(CellValue::Number(val))
}

fn fn_quartile(args: &[CellValue]) -> Result<CellValue, String> {
    let q = require_num!(args.last().unwrap());
    fn_percentile(&[args[..args.len()-1].to_vec(), vec![CellValue::Number(q / 4.0)]].concat())
}

fn fn_correl(args: &[CellValue]) -> Result<CellValue, String> {
    let n = args.len() / 2;
    let xs = nums_from_args(&args[..n]);
    let ys = nums_from_args(&args[n..]);
    let mx = xs.iter().sum::<f64>() / n as f64;
    let my = ys.iter().sum::<f64>() / n as f64;
    let cov: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| (x - mx) * (y - my)).sum();
    let sx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum::<f64>().sqrt();
    let sy: f64 = ys.iter().map(|y| (y - my).powi(2)).sum::<f64>().sqrt();
    Ok(CellValue::Number(cov / (sx * sy)))
}

fn fn_covar(args: &[CellValue]) -> Result<CellValue, String> {
    let n = args.len() / 2;
    let xs = nums_from_args(&args[..n]);
    let ys = nums_from_args(&args[n..]);
    let mx = xs.iter().sum::<f64>() / n as f64;
    let my = ys.iter().sum::<f64>() / n as f64;
    let cov: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| (x - mx) * (y - my)).sum::<f64>() / n as f64;
    Ok(CellValue::Number(cov))
}

fn fn_slope(args: &[CellValue]) -> Result<CellValue, String> {
    let n = args.len() / 2;
    let ys = nums_from_args(&args[..n]);
    let xs = nums_from_args(&args[n..]);
    let mx = xs.iter().sum::<f64>() / n as f64;
    let my = ys.iter().sum::<f64>() / n as f64;
    let num: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    Ok(CellValue::Number(num / den))
}

fn fn_intercept(args: &[CellValue]) -> Result<CellValue, String> {
    let n = args.len() / 2;
    let ys = nums_from_args(&args[..n]);
    let xs = nums_from_args(&args[n..]);
    let mx = xs.iter().sum::<f64>() / n as f64;
    let my = ys.iter().sum::<f64>() / n as f64;
    let slope = match fn_slope(args)? { CellValue::Number(n) => n, _ => return Ok(CellValue::Error(CellError::Value)) };
    Ok(CellValue::Number(my - slope * mx))
}

// ─── Text ─────────────────────────────────────────────────────────────────────

fn fn_len(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(require_text!(&args[0]).chars().count() as f64))
}

fn fn_left(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    let n = if args.len() > 1 { require_num!(&args[1]) as usize } else { 1 };
    Ok(CellValue::Text(s.chars().take(n).collect()))
}

fn fn_right(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    let n = if args.len() > 1 { require_num!(&args[1]) as usize } else { 1 };
    let chars: Vec<char> = s.chars().collect();
    let start = chars.len().saturating_sub(n);
    Ok(CellValue::Text(chars[start..].iter().collect()))
}

fn fn_mid(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    let start = (require_num!(&args[1]) as usize).saturating_sub(1);
    let len = require_num!(&args[2]) as usize;
    Ok(CellValue::Text(s.chars().skip(start).take(len).collect()))
}

fn fn_upper(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Text(require_text!(&args[0]).to_uppercase()))
}

fn fn_lower(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Text(require_text!(&args[0]).to_lowercase()))
}

fn fn_proper(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    let result = s.chars().enumerate().map(|(i, c)| {
        if i == 0 || !s.chars().nth(i-1).map(|p| p.is_alphanumeric()).unwrap_or(false) {
            c.to_uppercase().next().unwrap_or(c)
        } else {
            c.to_lowercase().next().unwrap_or(c)
        }
    }).collect();
    Ok(CellValue::Text(result))
}

fn fn_trim(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    // Trim leading/trailing whitespace and collapse internal spaces
    let trimmed = s.split_whitespace().collect::<Vec<&str>>().join(" ");
    Ok(CellValue::Text(trimmed))
}

fn fn_ltrim(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Text(require_text!(&args[0]).trim_start().to_string()))
}

fn fn_rtrim(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Text(require_text!(&args[0]).trim_end().to_string()))
}

fn fn_concatenate(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Text(args.iter().map(|v| v.as_text()).collect()))
}

fn fn_concat(args: &[CellValue]) -> Result<CellValue, String> {
    fn_concatenate(args)
}

fn fn_textjoin(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 3 { return Err("TEXTJOIN needs 3+ args".into()); }
    let delim = require_text!(&args[0]);
    let ignore_empty = to_bool(&args[1]);
    let parts: Vec<String> = args[2..].iter()
        .map(|v| v.as_text())
        .filter(|s| !ignore_empty || !s.is_empty())
        .collect();
    Ok(CellValue::Text(parts.join(&delim)))
}

fn fn_replace(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    let start = (require_num!(&args[1]) as usize).saturating_sub(1);
    let num_chars = require_num!(&args[2]) as usize;
    let new_text = require_text!(&args[3]);
    let chars: Vec<char> = s.chars().collect();
    let result: String = chars[..start.min(chars.len())].iter()
        .chain(new_text.chars().collect::<Vec<_>>().iter())
        .chain(chars[(start + num_chars).min(chars.len())..].iter())
        .collect();
    Ok(CellValue::Text(result))
}

fn fn_substitute(args: &[CellValue]) -> Result<CellValue, String> {
    let text = require_text!(&args[0]);
    let old = require_text!(&args[1]);
    let new = require_text!(&args[2]);
    let instance = if args.len() > 3 { Some(require_num!(&args[3]) as usize) } else { None };
    match instance {
        None => Ok(CellValue::Text(text.replace(old.as_str(), new.as_str()))),
        Some(n) => {
            let mut count = 0;
            let result = text.split(old.as_str()).enumerate().map(|(i, part)| {
                if i == 0 { part.to_string() }
                else { count += 1; if count == n { format!("{}{}", new, part) } else { format!("{}{}", old, part) } }
            }).collect();
            Ok(CellValue::Text(result))
        }
    }
}

fn fn_find(args: &[CellValue]) -> Result<CellValue, String> {
    let find = require_text!(&args[0]);
    let within = require_text!(&args[1]);
    let start = if args.len() > 2 { (require_num!(&args[2]) as usize).saturating_sub(1) } else { 0 };
    within[start..].find(find.as_str())
        .map(|pos| CellValue::Number((start + pos + 1) as f64))
        .ok_or_else(|| "Not found".into())
}

fn fn_search(args: &[CellValue]) -> Result<CellValue, String> {
    let find = require_text!(&args[0]).to_lowercase();
    let within = require_text!(&args[1]).to_lowercase();
    let start = if args.len() > 2 { (require_num!(&args[2]) as usize).saturating_sub(1) } else { 0 };
    within[start..].find(find.as_str())
        .map(|pos| CellValue::Number((start + pos + 1) as f64))
        .ok_or_else(|| "Not found".into())
}

fn fn_text(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let fmt = require_text!(&args[1]);
    // Basic number formatting
    let formatted = if fmt.contains('%') {
        format!("{:.0}%", n * 100.0)
    } else if fmt.contains('.') {
        let places = fmt.chars().rev().take_while(|c| *c == '0').count();
        format!("{:.prec$}", n, prec = places)
    } else {
        format!("{:.0}", n)
    };
    Ok(CellValue::Text(formatted))
}

fn fn_value(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    s.trim().replace(',', "").parse::<f64>()
        .map(CellValue::Number)
        .map_err(|_| format!("Cannot convert '{}' to number", s))
}

fn fn_fixed(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]);
    let places = if args.len() > 1 { require_num!(&args[1]) as usize } else { 2 };
    Ok(CellValue::Text(format!("{:.prec$}", n, prec = places)))
}

fn fn_rept(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    let n = require_num!(&args[1]) as usize;
    Ok(CellValue::Text(s.repeat(n)))
}

fn fn_code(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    s.chars().next().map(|c| CellValue::Number(c as u32 as f64))
        .ok_or_else(|| "Empty string".into())
}

fn fn_char(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]) as u32;
    char::from_u32(n).map(|c| CellValue::Text(c.to_string()))
        .ok_or_else(|| "Invalid char code".into())
}

fn fn_exact(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(require_text!(&args[0]) == require_text!(&args[1])))
}

fn fn_t(args: &[CellValue]) -> Result<CellValue, String> {
    match &args[0] {
        CellValue::Text(s) => Ok(CellValue::Text(s.clone())),
        _ => Ok(CellValue::Text(String::new())),
    }
}

// ─── Logical ─────────────────────────────────────────────────────────────────

fn fn_if(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 2 { return Err("IF needs 2+ args".into()); }
    if to_bool(&args[0]) {
        Ok(args[1].clone())
    } else {
        Ok(args.get(2).cloned().unwrap_or(CellValue::Bool(false)))
    }
}

fn fn_ifs(args: &[CellValue]) -> Result<CellValue, String> {
    for chunk in args.chunks(2) {
        if to_bool(&chunk[0]) {
            return Ok(chunk.get(1).cloned().unwrap_or(CellValue::Empty));
        }
    }
    Ok(CellValue::Error(CellError::NA))
}

fn fn_and(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(args.iter().all(to_bool)))
}

fn fn_or(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(args.iter().any(to_bool)))
}

fn fn_not(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(!to_bool(&args[0])))
}

fn fn_xor(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(args.iter().filter(|v| to_bool(v)).count() % 2 == 1))
}

fn fn_iferror(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 2 { return Err("IFERROR needs 2 args".into()); }
    if args[0].is_error() { Ok(args[1].clone()) } else { Ok(args[0].clone()) }
}

fn fn_ifna(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 2 { return Err("IFNA needs 2 args".into()); }
    if matches!(args[0], CellValue::Error(CellError::NA)) { Ok(args[1].clone()) } else { Ok(args[0].clone()) }
}

fn fn_switch(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 3 { return Err("SWITCH needs 3+ args".into()); }
    let expr = &args[0];
    let pairs = &args[1..];
    for chunk in pairs.chunks(2) {
        if chunk.len() == 1 { return Ok(chunk[0].clone()); } // default
        if &chunk[0] == expr { return Ok(chunk[1].clone()); }
    }
    Ok(CellValue::Error(CellError::NA))
}

fn fn_true(_: &[CellValue]) -> Result<CellValue, String> { Ok(CellValue::Bool(true)) }
fn fn_false(_: &[CellValue]) -> Result<CellValue, String> { Ok(CellValue::Bool(false)) }

// ─── Lookup ──────────────────────────────────────────────────────────────────

fn fn_vlookup(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 3 { return Err("VLOOKUP needs 3+ args".into()); }
    let lookup = &args[0];
    let col_idx = require_num!(&args[2]) as usize;
    // args[1] would be the range — simplified to searching the remaining args
    let data = &args[1..args.len()-1];
    for chunk in data.chunks(col_idx) {
        if chunk.first() == Some(lookup) {
            return Ok(chunk.get(col_idx - 1).cloned().unwrap_or(CellValue::Error(CellError::Ref)));
        }
    }
    Ok(CellValue::Error(CellError::NA))
}

fn fn_hlookup(args: &[CellValue]) -> Result<CellValue, String> {
    fn_vlookup(args) // simplified
}

fn fn_index(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 3 { return Err("INDEX needs 3 args".into()); }
    let row = require_num!(&args[1]) as usize;
    let col = require_num!(&args[2]) as usize;
    // Simplified: if args[0] is just values, treat as 1D
    args.get(row.max(1) - 1).cloned().ok_or_else(|| "INDEX out of bounds".into())
}

fn fn_match(args: &[CellValue]) -> Result<CellValue, String> {
    if args.len() < 2 { return Err("MATCH needs 2+ args".into()); }
    let lookup = &args[0];
    let data = &args[1..];
    data.iter().position(|v| v == lookup)
        .map(|i| CellValue::Number((i + 1) as f64))
        .ok_or_else(|| "Not found".into())
}

fn fn_choose(args: &[CellValue]) -> Result<CellValue, String> {
    let idx = require_num!(&args[0]) as usize;
    args.get(idx).cloned().ok_or_else(|| "CHOOSE index out of range".into())
}

fn fn_offset(args: &[CellValue]) -> Result<CellValue, String> {
    // Simplified stub
    Ok(args.first().cloned().unwrap_or(CellValue::Empty))
}

fn fn_row(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(1.0)) // simplified
}

fn fn_column(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(1.0)) // simplified
}

fn fn_rows(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(args.len() as f64))
}

fn fn_columns(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(1.0)) // simplified
}

fn fn_transpose(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(args.first().cloned().unwrap_or(CellValue::Empty))
}

// ─── Date & Time ─────────────────────────────────────────────────────────────

fn fn_now(_: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Text(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()))
}

fn fn_today(_: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Text(chrono::Utc::now().format("%Y-%m-%d").to_string()))
}

fn fn_date(args: &[CellValue]) -> Result<CellValue, String> {
    let y = require_num!(&args[0]) as i32;
    let m = require_num!(&args[1]) as u32;
    let d = require_num!(&args[2]) as u32;
    use chrono::NaiveDate;
    NaiveDate::from_ymd_opt(y, m, d)
        .map(|dt| CellValue::Date(dt))
        .ok_or_else(|| "Invalid date".into())
}

fn fn_time(args: &[CellValue]) -> Result<CellValue, String> {
    let h = require_num!(&args[0]);
    let m = require_num!(&args[1]);
    let s = require_num!(&args[2]);
    Ok(CellValue::Number((h * 3600.0 + m * 60.0 + s) / 86400.0))
}

fn extract_date(v: &CellValue) -> Option<chrono::NaiveDate> {
    match v {
        CellValue::Date(d) => Some(*d),
        CellValue::Text(s) => s.parse().ok(),
        _ => None,
    }
}

fn fn_year(args: &[CellValue]) -> Result<CellValue, String> {
    use chrono::Datelike;
    extract_date(&args[0]).map(|d| CellValue::Number(d.year() as f64))
        .ok_or_else(|| "Invalid date".into())
}
fn fn_month(args: &[CellValue]) -> Result<CellValue, String> {
    use chrono::Datelike;
    extract_date(&args[0]).map(|d| CellValue::Number(d.month() as f64))
        .ok_or_else(|| "Invalid date".into())
}
fn fn_day(args: &[CellValue]) -> Result<CellValue, String> {
    use chrono::Datelike;
    extract_date(&args[0]).map(|d| CellValue::Number(d.day() as f64))
        .ok_or_else(|| "Invalid date".into())
}
fn fn_hour(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(0.0)) // simplified
}
fn fn_minute(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(0.0))
}
fn fn_second(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(0.0))
}
fn fn_weekday(args: &[CellValue]) -> Result<CellValue, String> {
    use chrono::Datelike;
    extract_date(&args[0]).map(|d| CellValue::Number(d.weekday().num_days_from_sunday() as f64 + 1.0))
        .ok_or_else(|| "Invalid date".into())
}
fn fn_weeknum(args: &[CellValue]) -> Result<CellValue, String> {
    use chrono::Datelike;
    extract_date(&args[0]).map(|d| CellValue::Number(d.iso_week().week() as f64))
        .ok_or_else(|| "Invalid date".into())
}
fn fn_edate(args: &[CellValue]) -> Result<CellValue, String> {
    use chrono::Datelike;
    let d = extract_date(&args[0]).ok_or_else(|| "Invalid date".to_string())?;
    let months = require_num!(&args[1]) as i32;
    let new_month = d.month() as i32 + months;
    let year_add = (new_month - 1) / 12;
    let month = ((new_month - 1) % 12 + 12) % 12 + 1;
    let new_date = chrono::NaiveDate::from_ymd_opt(d.year() + year_add, month as u32, d.day())
        .ok_or_else(|| "Invalid date result".to_string())?;
    Ok(CellValue::Date(new_date))
}
fn fn_eomonth(args: &[CellValue]) -> Result<CellValue, String> {
    use chrono::Datelike;
    let d = extract_date(&args[0]).ok_or_else(|| "Invalid date".to_string())?;
    let months = require_num!(&args[1]) as i32;
    let new_month = d.month() as i32 + months + 1;
    let year_add = (new_month - 1) / 12;
    let month = ((new_month - 1) % 12 + 12) % 12 + 1;
    let first_next = chrono::NaiveDate::from_ymd_opt(d.year() + year_add, month as u32, 1).unwrap();
    let last = first_next - chrono::Duration::days(1);
    Ok(CellValue::Date(last))
}
fn fn_networkdays(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(0.0)) // simplified stub
}
fn fn_workday(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(args[0].clone()) // simplified stub
}
fn fn_datedif(args: &[CellValue]) -> Result<CellValue, String> {
    let start = extract_date(&args[0]).ok_or_else(|| "Invalid start date".to_string())?;
    let end = extract_date(&args[1]).ok_or_else(|| "Invalid end date".to_string())?;
    let unit = require_text!(&args[2]).to_uppercase();
    let diff = (end - start).num_days();
    match unit.as_str() {
        "D" => Ok(CellValue::Number(diff as f64)),
        "M" => {
            use chrono::Datelike;
            let months = (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
            Ok(CellValue::Number(months as f64))
        }
        "Y" => {
            use chrono::Datelike;
            Ok(CellValue::Number((end.year() - start.year()) as f64))
        }
        _ => Ok(CellValue::Number(diff as f64)),
    }
}
fn fn_datevalue(args: &[CellValue]) -> Result<CellValue, String> {
    let s = require_text!(&args[0]);
    s.parse::<chrono::NaiveDate>()
        .map(|d| CellValue::Date(d))
        .map_err(|_| format!("Cannot parse date: {}", s))
}
fn fn_timevalue(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(0.0)) // simplified
}

// ─── Information ─────────────────────────────────────────────────────────────

fn fn_isnumber(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(args[0].is_number()))
}
fn fn_istext(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(args[0].is_text()))
}
fn fn_isblank(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(args[0].is_empty()))
}
fn fn_iserror(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(args[0].is_error()))
}
fn fn_iserr(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(matches!(&args[0], CellValue::Error(e) if !matches!(e, CellError::NA))))
}
fn fn_isna(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(matches!(&args[0], CellValue::Error(CellError::NA))))
}
fn fn_islogical(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Bool(matches!(args[0], CellValue::Bool(_))))
}
fn fn_isodd(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]) as i64;
    Ok(CellValue::Bool(n % 2 != 0))
}
fn fn_iseven(args: &[CellValue]) -> Result<CellValue, String> {
    let n = require_num!(&args[0]) as i64;
    Ok(CellValue::Bool(n % 2 == 0))
}
fn fn_na(_: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Error(CellError::NA))
}
fn fn_error_type(args: &[CellValue]) -> Result<CellValue, String> {
    match &args[0] {
        CellValue::Error(e) => Ok(CellValue::Number(match e {
            CellError::Null => 1.0, CellError::Div0 => 2.0, CellError::Value => 3.0,
            CellError::Ref => 4.0, CellError::Name => 5.0, CellError::Num => 6.0,
            CellError::NA => 7.0, _ => 8.0,
        })),
        _ => Ok(CellValue::Error(CellError::NA)),
    }
}
fn fn_cell(_: &[CellValue]) -> Result<CellValue, String> { Ok(CellValue::Text(String::new())) }
fn fn_type(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(match &args[0] {
        CellValue::Number(_) => 1.0,
        CellValue::Text(_) => 2.0,
        CellValue::Bool(_) => 4.0,
        CellValue::Error(_) => 16.0,
        CellValue::Empty => 1.0,
        _ => 1.0,
    }))
}
fn fn_n(args: &[CellValue]) -> Result<CellValue, String> {
    Ok(CellValue::Number(args[0].as_number().unwrap_or(0.0)))
}

// ─── Financial ───────────────────────────────────────────────────────────────

fn fn_pmt(args: &[CellValue]) -> Result<CellValue, String> {
    let rate = require_num!(&args[0]);
    let nper = require_num!(&args[1]);
    let pv = require_num!(&args[2]);
    let fv = if args.len() > 3 { require_num!(&args[3]) } else { 0.0 };
    let type_ = if args.len() > 4 { require_num!(&args[4]) } else { 0.0 };
    let pmt = if rate == 0.0 {
        -(pv + fv) / nper
    } else {
        let factor = (1.0 + rate).powf(nper);
        -(pv * factor * rate + fv * rate) / ((factor - 1.0) * (1.0 + rate * type_))
    };
    Ok(CellValue::Number(pmt))
}

fn fn_pv(args: &[CellValue]) -> Result<CellValue, String> {
    let rate = require_num!(&args[0]);
    let nper = require_num!(&args[1]);
    let pmt = require_num!(&args[2]);
    let fv = if args.len() > 3 { require_num!(&args[3]) } else { 0.0 };
    let pv = if rate == 0.0 {
        -(pmt * nper + fv)
    } else {
        let factor = (1.0 + rate).powf(nper);
        -(pmt * (factor - 1.0) / rate + fv) / factor
    };
    Ok(CellValue::Number(pv))
}

fn fn_fv(args: &[CellValue]) -> Result<CellValue, String> {
    let rate = require_num!(&args[0]);
    let nper = require_num!(&args[1]);
    let pmt = require_num!(&args[2]);
    let pv = if args.len() > 3 { require_num!(&args[3]) } else { 0.0 };
    let fv = if rate == 0.0 {
        -(pmt * nper + pv)
    } else {
        let factor = (1.0 + rate).powf(nper);
        -(pmt * (factor - 1.0) / rate + pv * factor)
    };
    Ok(CellValue::Number(fv))
}

fn fn_rate(args: &[CellValue]) -> Result<CellValue, String> {
    // Newton-Raphson iteration for RATE
    let nper = require_num!(&args[0]);
    let pmt = require_num!(&args[1]);
    let pv = require_num!(&args[2]);
    let mut rate = 0.1;
    for _ in 0..1000 {
        let f = pv * (1.0_f64 + rate).powf(nper) + pmt * ((1.0_f64 + rate).powf(nper) - 1.0) / rate;
        let df = pv * nper * (1.0_f64 + rate).powf(nper - 1.0) +
            pmt * (nper * (1.0_f64 + rate).powf(nper - 1.0) * rate - ((1.0_f64 + rate).powf(nper) - 1.0)) / (rate * rate);
        let new_rate = rate - f / df;
        if (new_rate - rate).abs() < 1e-10 { return Ok(CellValue::Number(new_rate)); }
        rate = new_rate;
    }
    Ok(CellValue::Number(rate))
}

fn fn_nper(args: &[CellValue]) -> Result<CellValue, String> {
    let rate = require_num!(&args[0]);
    let pmt = require_num!(&args[1]);
    let pv = require_num!(&args[2]);
    if rate == 0.0 { return Ok(CellValue::Number(-pv / pmt)); }
    let nper = ((pmt / rate) / (pmt / rate + pv)).log(1.0 + rate);
    Ok(CellValue::Number(nper))
}

fn fn_npv(args: &[CellValue]) -> Result<CellValue, String> {
    let rate = require_num!(&args[0]);
    let npv: f64 = args[1..].iter()
        .enumerate()
        .filter_map(|(i, v)| v.as_number().map(|n| n / (1.0 + rate).powi(i as i32 + 1)))
        .sum();
    Ok(CellValue::Number(npv))
}

fn fn_irr(args: &[CellValue]) -> Result<CellValue, String> {
    if args.is_empty() { return Err("IRR needs cash flows".into()); }
    let flows: Vec<f64> = args.iter().filter_map(|v| v.as_number()).collect();
    let mut rate = 0.1;
    for _ in 0..1000 {
        let npv: f64 = flows.iter().enumerate()
            .map(|(i, cf)| cf / (1.0_f64 + rate).powi(i as i32))
            .sum();
        let d_npv: f64 = flows.iter().enumerate()
            .map(|(i, cf)| -(i as f64) * cf / (1.0_f64 + rate).powi(i as i32 + 1))
            .sum();
        if d_npv.abs() < 1e-15 { break; }
        let new_rate = rate - npv / d_npv;
        if (new_rate - rate).abs() < 1e-10 { return Ok(CellValue::Number(new_rate)); }
        rate = new_rate;
    }
    Ok(CellValue::Number(rate))
}

fn fn_ipmt(args: &[CellValue]) -> Result<CellValue, String> {
    let rate = require_num!(&args[0]);
    let per = require_num!(&args[1]);
    let nper = require_num!(&args[2]);
    let pv = require_num!(&args[3]);
    let pmt_val = match fn_pmt(&[CellValue::Number(rate), CellValue::Number(nper), CellValue::Number(pv)])? {
        CellValue::Number(n) => n,
        _ => return Ok(CellValue::Error(CellError::Value)),
    };
    let ipmt = -(pv * (1.0_f64 + rate).powf(per - 1.0) * rate + pmt_val * ((1.0_f64 + rate).powf(per - 1.0) - 1.0));
    Ok(CellValue::Number(ipmt))
}

fn fn_ppmt(args: &[CellValue]) -> Result<CellValue, String> {
    let pmt_val = match fn_pmt(&args[..4])? { CellValue::Number(n) => n, _ => return Ok(CellValue::Error(CellError::Value)) };
    let ipmt_val = match fn_ipmt(args)? { CellValue::Number(n) => n, _ => return Ok(CellValue::Error(CellError::Value)) };
    Ok(CellValue::Number(pmt_val - ipmt_val))
}
