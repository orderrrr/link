use std::fmt;

use crate::{vm::bool_to_int, Pair};

type Fmt<'a> = fmt::Formatter<'a>;
type Res = Result<(), fmt::Error>;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum FN /*monadic operator*/ {
    Plus,
    Minus,
    Mult,
    Div,
    Max,
    Min,
    Eq,
    Amp,
    Bang,
    Rho,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum CN /*combinators*/ {
    Fold,
    ScanL,
    Each,
}

impl FN {
    pub fn as_str(&self) -> &'static str {
        match self {
            FN::Plus => "+",
            FN::Minus => "-",
            FN::Mult => "×",
            FN::Div => "÷",
            FN::Max => "¯",
            FN::Min => "_",
            FN::Eq => "=",
            FN::Amp => "&",
            FN::Bang => "!",
            FN::Rho => "ρ",
        }
    }

    pub fn from_string(s: &str) -> FN {
        match s {
            "+" => FN::Plus,
            "-" => FN::Minus,
            "×" => FN::Mult,
            "÷" => FN::Div,
            "¯" => FN::Max,
            "_" => FN::Min,
            "=" => FN::Eq,
            "&" => FN::Amp,
            "!" => FN::Bang,
            "ρ" => FN::Rho,
            _ => unreachable!("Unknown FN: {}", s),
        }
    }
}

impl fmt::Display for FN {
    fn fmt(&self, f: &mut Fmt<'_>) -> Res {
        write!(f, "{}", self.as_str())
    }
}

impl CN {
    pub fn as_str(&self) -> &'static str {
        match self {
            CN::Fold => "/",
            CN::ScanL => "\\",
            CN::Each => "ǁ",
        }
    }
    pub fn from_string(s: &str) -> CN {
        match s {
            "/" => CN::Fold,
            "\\" => CN::ScanL,
            "ǁ" => CN::Each,
            _ => unreachable!("Unknown CN: {}", s),
        }
    }
}

impl fmt::Display for CN {
    fn fmt(&self, f: &mut Fmt<'_>) -> Res {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialOrd)]
pub struct NN {
    pub n: E,
    pub start: usize,
    pub end: usize,
}

impl PartialEq for NN {
    fn eq(&self, other: &Self) -> bool {
        match (self.clone().n, other.clone().n) {
            (E::BOOL(w), E::INT(a)) | (E::INT(a), E::BOOL(w)) => bool_to_int(w) == a,
            _ => self.n == other.n,
        }
    }
}

impl NN {
    pub fn new(p: Pair, n: E) -> NN {
        let s = p.as_span();
        NN {
            n,
            start: s.start(),
            end: s.end(),
        }
    }
    pub fn ndb(n: E) -> Box<NN> {
        Box::new(NN {
            n,
            start: 0,
            end: 0,
        })
    }
    pub fn nd(n: E) -> NN {
        NN {
            n,
            start: 0,
            end: 0,
        }
    }
    pub fn ndv(n: E) -> Vec<NN> {
        vec![NN {
            n,
            start: 0,
            end: 0,
        }]
    }
}

impl fmt::Display for NN {
    fn fmt(&self, f: &mut Fmt<'_>) -> Res {
        write!(f, "{}", self.n)
    }
}

// ---------------------------------------------------------------------------
// AST expression enum — S-expression based
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum E {
    // Literals
    INT(i32),
    BOOL(bool),
    FT(f64),
    ST(String),

    // Names
    VAL(String), // variable reference

    // Operators and combinators (as AST nodes in trains)
    MFN(FN), // monadic function
    DFN(FN), // dyadic function
    CN(CN),  // combinator

    // Operator + combinator pair (e.g. +/)
    MCO {
        o: Box<NN>,
        co: Box<NN>,
    },
    DCO {
        o: Box<NN>,
        co: Box<NN>,
    },

    // Monadic-override marker (op followed by :)
    MOP(FN),

    // List / array
    LIST(Vec<NN>),

    // Application: train applied to arguments
    // 1 arg = monadic, 2 args = dyadic
    APPLY {
        train: Vec<NN>, // the operator/combinator chain
        args: Vec<NN>,  // the arguments
    },

    // Lambda: (λ (params...) body...)
    LAMBDA {
        params: Vec<String>,
        body: Vec<NN>,
    },

    // Do-block: (↻ expr1 expr2 ... exprN) — evaluate all, return last
    DOBLOCK(Vec<NN>),

    // Assignment: (: name expr)
    ASEXP {
        name: String,
        rhs: Box<NN>,
    },

    // User function value (runtime representation of a compiled lambda)
    // Self-contained: carries its own bytecode and constants so it works
    // across REPL compilation units.
    UFNV {
        nparams: usize,    // number of parameters
        body_op: Vec<u8>,  // bytecode for the function body (STORE params + body + END)
        body_var: Vec<NN>, // constant pool entries the body references
    },
}

impl fmt::Display for E {
    fn fmt(&self, f: &mut Fmt<'_>) -> Res {
        match &self {
            E::INT(n) => write!(f, "{}", n),
            E::FT(n) => write!(f, "{}", n),
            E::BOOL(b) => write!(f, "{}", if *b { 1 } else { 0 }),
            E::ST(s) => write!(f, "\"{}\"", s),
            E::VAL(s) => write!(f, "{}", s),
            E::MFN(o) => write!(f, "{}", o),
            E::DFN(d) => write!(f, "{}", d),
            E::CN(co) => write!(f, "{}", co),
            E::DCO { o, co } | E::MCO { o, co } => write!(f, "{}{}", o, co),
            E::MOP(o) => write!(f, "{}:", o),
            E::LIST(t) => fmt_list(f, t),
            E::APPLY { train, args } => {
                write!(f, "(")?;
                for t in train {
                    write!(f, "{}", t)?;
                }
                for a in args {
                    write!(f, " {}", a)?;
                }
                write!(f, ")")
            }
            E::LAMBDA { params, body } => {
                write!(f, "(λ (")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")")?;
                for b in body {
                    write!(f, " {}", b)?;
                }
                write!(f, ")")
            }
            E::DOBLOCK(exprs) => {
                write!(f, "(↻")?;
                for e in exprs {
                    write!(f, " {}", e)?;
                }
                write!(f, ")")
            }
            E::ASEXP { name, rhs } => write!(f, "(: {} {})", name, rhs),
            E::UFNV { nparams, .. } => write!(f, "<fn:{}>", nparams),
        }
    }
}

fn fmt_list(f: &mut Fmt<'_>, t: &Vec<NN>) -> Res {
    if t.is_empty() {
        return write!(f, "()");
    }

    // Check if this is a 2D structure (all elements are lists)
    let all_lists = t.iter().all(|el| matches!(&el.n, E::LIST(_)));

    if !all_lists {
        // Rank 1: space-separated on one line
        for (i, el) in t.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", el)?;
        }
        return Ok(());
    }

    // Rank 2+: grid format with right-aligned columns
    let rows: Vec<Vec<String>> = t
        .iter()
        .map(|row| match &row.n {
            E::LIST(inner) => inner.iter().map(|el| format!("{}", el)).collect(),
            _ => vec![format!("{}", row)],
        })
        .collect();

    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut col_widths = vec![0usize; max_cols];
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(cell.len());
        }
    }

    for (row_idx, row) in rows.iter().enumerate() {
        if row_idx > 0 {
            writeln!(f)?;
        }
        for (col_idx, cell) in row.iter().enumerate() {
            if col_idx > 0 {
                write!(f, " ")?;
            }
            write!(f, "{:>width$}", cell, width = col_widths[col_idx])?;
        }
    }

    Ok(())
}

pub fn gen_sep(sep: &str, r: &str, n: &NN) -> String {
    if r.eq("") {
        format!("{}", n)
    } else {
        format!("{}{}{}", r, sep, n)
    }
}

pub fn om(o: FN) -> E {
    E::MFN(o)
}

pub fn od(o: FN) -> E {
    E::DFN(o)
}

pub fn oc(c: CN) -> E {
    E::CN(c)
}
