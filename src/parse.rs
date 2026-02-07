use crate::{
    ast::{CN, E, FN, NN},
    err::{LErrEnum as LErrE, LErrEnum::ExprExpected as Er},
    LocatedError, Res, Rule, VRes, LP,
};

use debug_print::debug_println;
use pest::Parser;

pub type Pair<'i> = pest::iterators::Pair<'i, Rule>;

/// Parse a source string into a Vec of AST nodes.
/// In the new S-expression syntax, the program is a single top-level sexpr.
pub fn parse(source: &str) -> VRes {
    let pairs = LP::parse(Rule::prg, source)?;
    let ast: VRes = pairs
        .into_iter()
        .filter(|p| p.as_rule() == Rule::sexpr)
        .map(build_sexpr)
        .collect();
    ast
}

/// Convenience: parse and return the first expression.
pub fn pf(source: &str) -> NN {
    parse(source).unwrap().first().unwrap().to_owned()
}

/// Build an AST node from a sexpr pair.
/// A sexpr is either an atom or a parenthesized (inner).
fn build_sexpr(pair: Pair) -> Res {
    debug_println!(
        "build_sexpr: rule={:?} str={}",
        pair.as_rule(),
        pair.as_str()
    );
    match pair.as_rule() {
        Rule::sexpr => {
            // sexpr = { "(" ~ inner ~ ")" | atom }
            // inner is silent, so we see its children directly
            let inner = pair.clone().into_inner().next().ok_or(Er)?;
            match inner.as_rule() {
                // If the inner is one of the named rules, build accordingly
                Rule::lambda => build_lambda(pair, inner),
                Rule::doblock => build_doblock(pair, inner),
                Rule::assign => build_assign(pair, inner),
                Rule::apply => build_apply(pair, inner),
                Rule::list_inner => build_list(pair, inner),
                // Atoms
                Rule::int => build_int(inner),
                Rule::float => build_float(inner),
                Rule::string => build_string(inner),
                Rule::ident => build_ident(inner),
                // Nested sexpr (shouldn't happen since inner is silent, but just in case)
                Rule::sexpr => build_sexpr(inner),
                _ => Err(LocatedError::from(LErrE::ExprExpected)),
            }
        }
        // Direct atoms (when sexpr matched the atom alternative)
        Rule::int => build_int(pair),
        Rule::float => build_float(pair),
        Rule::string => build_string(pair),
        Rule::ident => build_ident(pair),
        _ => Err(LocatedError::from(LErrE::ExprExpected)),
    }
}

/// (λ (params...) body...)
fn build_lambda(outer: Pair, pair: Pair) -> Res {
    let mut inner = pair.into_inner();

    // First child is params: (ident*)
    let params_pair = inner.next().ok_or(Er)?;
    let params: Vec<String> = params_pair
        .into_inner()
        .map(|p| p.as_str().to_string())
        .collect();

    // Remaining children are body sexprs
    let body: Result<Vec<NN>, _> = inner.map(build_sexpr).collect();

    Ok(NN::new(
        outer,
        E::LAMBDA {
            params,
            body: body?,
        },
    ))
}

/// (↻ expr1 expr2 ... exprN)
fn build_doblock(outer: Pair, pair: Pair) -> Res {
    let exprs: Result<Vec<NN>, _> = pair.into_inner().map(build_sexpr).collect();
    Ok(NN::new(outer, E::DOBLOCK(exprs?)))
}

/// (: name expr)
fn build_assign(outer: Pair, pair: Pair) -> Res {
    let mut inner = pair.into_inner();
    let name = inner.next().ok_or(Er)?.as_str().to_string();
    let rhs = build_sexpr(inner.next().ok_or(Er)?)?;

    Ok(NN::new(
        outer,
        E::ASEXP {
            name,
            rhs: Box::new(rhs),
        },
    ))
}

/// (train args...)
/// train is a sequence of ops/combinators/names
fn build_apply(outer: Pair, pair: Pair) -> Res {
    let mut inner = pair.into_inner();

    // First child is the train
    let train_pair = inner.next().ok_or(Er)?;
    let train = build_train(train_pair)?;

    // Remaining children are args
    let args: Result<Vec<NN>, _> = inner.map(build_sexpr).collect();

    Ok(NN::new(outer, E::APPLY { train, args: args? }))
}

/// Build train elements from a train pair.
/// A train is: (mop | cfn | op | cn | ident)+
fn build_train(pair: Pair) -> Result<Vec<NN>, LocatedError<LErrE>> {
    let mut elems = Vec::new();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::op => {
                elems.push(NN::new(
                    child.clone(),
                    E::MFN(FN::from_string(child.as_str())),
                ));
            }
            Rule::cn => {
                elems.push(NN::new(
                    child.clone(),
                    E::CN(CN::from_string(child.as_str())),
                ));
            }
            Rule::mop => {
                // mop = { op ~ ":" } — monadic override
                let op_pair = child.clone().into_inner().next().ok_or(Er)?;
                elems.push(NN::new(child, E::MOP(FN::from_string(op_pair.as_str()))));
            }
            Rule::cfn => {
                // cfn = { (op | ident) ~ cn }
                let mut cfn_inner = child.clone().into_inner();
                let op_part = cfn_inner.next().ok_or(Er)?;
                let cn_part = cfn_inner.next().ok_or(Er)?;

                let o: Box<NN> = match op_part.as_rule() {
                    Rule::op => Box::new(NN::new(
                        op_part.clone(),
                        E::MFN(FN::from_string(op_part.as_str())),
                    )),
                    Rule::ident => Box::new(NN::new(
                        op_part.clone(),
                        E::VAL(op_part.as_str().to_string()),
                    )),
                    _ => return Err(LocatedError::from(Er)),
                };
                let co = Box::new(NN::new(
                    cn_part.clone(),
                    E::CN(CN::from_string(cn_part.as_str())),
                ));
                // In a train context we don't know arity yet — use MCO
                // The compiler/VM will resolve based on actual arg count
                elems.push(NN::new(child, E::MCO { o, co }));
            }
            Rule::ident => {
                elems.push(NN::new(child.clone(), E::VAL(child.as_str().to_string())));
            }
            _ => return Err(LocatedError::from(Er)),
        }
    }
    Ok(elems)
}

/// List literal: first element is a value, not an operator
/// (1 2 3) or ((1 2) (3 4))
fn build_list(outer: Pair, pair: Pair) -> Res {
    let elems: Result<Vec<NN>, _> = pair.into_inner().map(build_sexpr).collect();
    Ok(NN::new(outer, E::LIST(elems?)))
}

// ---------------------------------------------------------------------------
// Atom builders
// ---------------------------------------------------------------------------

fn build_int(pair: Pair) -> Res {
    let istr = pair.as_str();
    let integer: i32 = istr.parse()?;
    Ok(NN::new(pair, E::INT(integer)))
}

fn build_float(pair: Pair) -> Res {
    let fstr = pair.as_str();
    let flt: f64 = fstr.parse()?;
    Ok(NN::new(pair, E::FT(flt)))
}

fn build_string(pair: Pair) -> Res {
    let s = pair.as_str();
    // Strip surrounding quotes and unescape ""
    let s = &s[1..s.len() - 1];
    let s = s.replace("\"\"", "\"");
    Ok(NN::new(pair, E::ST(s)))
}

fn build_ident(pair: Pair) -> Res {
    Ok(NN::new(pair.clone(), E::VAL(pair.as_str().to_string())))
}
