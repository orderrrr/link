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
pub enum CN /*compinators*/ {
    Fold,
    ScanL,
    Each,
}

impl FN {
    fn as_str(&self) -> &'static str {
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
    fn as_str(&self) -> &'static str {
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum E {
    INT(i32),
    BOOL(bool),
    FT(f64),
    ST(String),
    VAL(String),  // separate from S for formatting
    FVAL(String), // separate from value ( function value )

    MFN(FN),
    DFN(FN),
    CN(CN),

    MCO {
        o: Box<NN>,
        co: Box<NN>,
    },
    DCO {
        o: Box<NN>,
        co: Box<NN>,
    },

    BL(Box<NN>),
    FBLOCK(Vec<NN>),
    MFBLOCK(Vec<NN>),
    TFBLOCK(Vec<NN>),
    TMFBLOCK(Vec<NN>),
    LIST(Vec<NN>),

    MTRAIN(Vec<NN>),
    DTRAIN(Vec<NN>),

    MEXP {
        op: Vec<NN>,
        rhs: Box<NN>,
    },
    DEXP {
        op: Vec<NN>,
        lhs: Box<NN>,
        rhs: Box<NN>,
    },

    DDTRAIN {
        op: Vec<NN>,
        lhs: Box<NN>,
    },

    DBLOCK {
        op: Vec<NN>,
        lhs: Box<NN>,
    },

    ASEXP {
        op: Box<NN>,
        rhs: Box<NN>,
    },
}

impl E {
    fn write_train(op: &Vec<NN>) -> String {
        if op.len() > 1 {
            format!(
                "{}|",
                op.into_iter().fold(String::new(), |r, o| {
                    if r.eq("") {
                        format!("|{}", o.n)
                    } else {
                        format!("{}{}", r, o.n)
                    }
                })
            )
        } else {
            // safe to unwrap
            format!("{}", op.first().unwrap())
        }
    }
}

impl fmt::Display for E {
    fn fmt(&self, f: &mut Fmt<'_>) -> Res {
        match &self {
            E::INT(n) => write!(f, "{}", n),
            E::FT(n) => write!(f, "{}", n),
            E::BOOL(b) => write!(f, "{}", {
                match b {
                    true => 1,
                    false => 0,
                }
            }),
            E::ST(s) => write!(f, "\"{}\"", s),
            E::VAL(s) | E::FVAL(s) => write!(f, "{}", s),
            E::BL(b) => write!(f, "({})", b),
            E::MFN(mo) => write!(f, "{}", mo),
            E::DFN(d) => write!(f, "{}", d),
            E::CN(co) => write!(f, "{}", co),
            E::DCO { o, co } => write!(f, "{}{}", o, co),
            E::MCO { o, co } => write!(f, "{}{}", o, co),
            E::DTRAIN(tb) | E::MTRAIN(tb) => write!(f, "({})", {
                tb.into_iter()
                    .fold(String::new(), |r, t| gen_sep("", &r, t))
            }),
            E::MEXP { op, rhs } => write!(f, "{}{}", E::write_train(op), rhs),
            E::DEXP { op, lhs, rhs } => write!(f, "{}{}{}", lhs, E::write_train(op), rhs),
            E::DDTRAIN { op, lhs } => write!(f, "{}|{}", lhs, {
                op.into_iter()
                    .fold(String::new(), |r, t| gen_sep("", &r, t))
            }),
            E::DBLOCK { op, lhs } => write!(f, "{}|{}", lhs, {
                op.into_iter()
                    .fold(String::new(), |r, t| gen_sep("", &r, t))
            }),
            E::ASEXP { op, rhs } => write!(f, "{}:{}", op, rhs),
            E::FBLOCK(fb) | E::MFBLOCK(fb) => {
                write!(f, "{{{}}}", {
                    fb.into_iter()
                        .fold(String::new(), |r, t| gen_sep(";", &r, t))
                })
            }
            E::TMFBLOCK(fb) => {
                write!(f, "{{{}|}}", {
                    fb.into_iter()
                        .fold(String::new(), |r, t| gen_sep("", &r, t))
                })
            }
            E::TFBLOCK(fb) => {
                write!(f, "{{|{}|}}", {
                    fb.into_iter()
                        .fold(String::new(), |r, t| gen_sep("", &r, t))
                })
            }
            E::LIST(t) => fmt_list(f, t),
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
    // First, collect all rows as vectors of formatted strings
    let rows: Vec<Vec<String>> = t
        .iter()
        .map(|row| match &row.n {
            E::LIST(inner) => inner.iter().map(|el| format!("{}", el)).collect(),
            _ => vec![format!("{}", row)],
        })
        .collect();

    // Compute max width for each column index
    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut col_widths = vec![0usize; max_cols];
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(cell.len());
        }
    }

    // Print each row on its own line, right-aligned per column
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
