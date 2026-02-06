use crate::{
    ast::{CN, E, FN, NN},
    err::{err, LErrEnum as LErrE, LErrEnum::ExprExpected as Er},
    BoRes, LocatedError, PRes, Res, Rule, VRes, LP,
};

use debug_print::debug_println;
use pest::Parser;

pub type Pair<'i> = pest::iterators::Pair<'i, Rule>;

pub fn parse(source: &str) -> VRes {
    let ast: VRes = LP::parse(Rule::prg, source)?
        .into_iter()
        .filter(|p| p.as_rule() == Rule::exp)
        .map(|p| build_ast_from_expr(p, false))
        .collect();
    ast
}

pub fn pf(source: &str) -> NN {
    parse(source).unwrap().first().unwrap().to_owned()
}

fn parse_train(pair: Pair, monadic: bool) -> VRes {
    match pair.as_rule() {
        Rule::mop => Ok(vec![parse_fn(pair.into_inner().next().ok_or(Er)?, true)?]),
        Rule::fnvalue
        | Rule::texp
        | Rule::cfn
        | Rule::op
        | Rule::fnblock
        | Rule::fnblockm
        | Rule::fnblocko
        | Rule::dblock => Ok(vec![parse_fn(pair, monadic)?]),
        Rule::fn_list | Rule::block_exp => pair
            .into_inner()
            .map(|fun| parse_fn(fun, monadic))
            .collect(),
        Rule::EOI
        | Rule::prg
        | Rule::fun
        | Rule::block
        | Rule::exp_list
        | Rule::term
        | Rule::fnblockl
        | Rule::op_exp
        | Rule::nline
        | Rule::ascii
        | Rule::number
        | Rule::letter
        | Rule::WHITESPACE
        | Rule::COMMENT
        | Rule::exp
        | Rule::fun_d
        | Rule::fun_m
        | Rule::asexp
        | Rule::train
        | Rule::dtrain
        | Rule::cn
        | Rule::terms
        | Rule::listblock
        | Rule::fun_om
        | Rule::fun_dm
        | Rule::int
        | Rule::ft
        | Rule::value
        | Rule::string => Err(LocatedError::from(LErrE::TrainExpected(err(
            pair.as_str(),
            pair.as_span(),
        )))),
    }
}

fn parse_fn(pair: Pair, monadic: bool) -> Res {
    match pair.as_rule() {
        Rule::fnvalue => to_fnval(pair.clone(), pair.as_str()),
        Rule::op => match monadic {
            true => to_mfn(pair),
            false => to_dfn(pair),
        },
        Rule::mop => parse_fn(pair.into_inner().next().ok_or(Er)?, true),
        Rule::cn => to_cn(pair),
        Rule::dblock => {
            let (mtb, d_o, _) = grab_inner_exprs(pair.clone());
            let mtb = parse_fn(mtb.ok_or(Er)?, monadic);
            let d_o = parse_fn(d_o.ok_or(Er)?, false);

            to_dblock(pair, vec![d_o?], mtb?)
        }
        Rule::fnblock => {
            let ipair = pair.clone().into_inner();
            let list: VRes = ipair.map(|fun| build_ast_from_expr(fun, false)).collect();
            to_fblock(pair, list?)
        }
        Rule::fnblocko => {
            let mut ipair = pair.clone().into_inner();
            let train = parse_train(ipair.next().ok_or(Er)?, false);

            to_tfblock(pair, train?)
        }
        Rule::fnblockm => {
            let mut ipair = pair.clone().into_inner();
            let train = parse_train(ipair.next().ok_or(Er)?, true);

            to_tmfblock(pair, train?)
        }
        Rule::cfn => {
            let (o, co, _) = grab_inner_exprs(pair.clone());

            let o = Box::new(parse_fn(o.ok_or(Er)?, monadic)?);
            let co = Box::new(parse_fn(co.ok_or(Er)?, monadic)?);

            Ok(NN::new(
                pair,
                match monadic {
                    true => E::MCO { o, co },
                    false => E::DCO { o, co },
                },
            ))
        }
        Rule::dtrain => {
            let (lhs, op, _) = grab_inner_exprs(pair.clone());

            let lhs = build_ast_from_expr(lhs.ok_or(Er)?, false);
            let op: VRes = op
                .ok_or(Er)?
                .into_inner()
                .map(|op| parse_fn(op, false))
                .collect();
            to_ddtrain(pair, op?, lhs?)
        }
        Rule::block_exp => Ok(NN::new(
            pair.clone(),
            match monadic {
                true => E::MTRAIN(parse_train(pair, true)?),
                false => E::DTRAIN(parse_train(pair, false)?),
            },
        )),
        Rule::EOI
        | Rule::prg
        | Rule::op_exp
        | Rule::exp_list
        | Rule::term
        | Rule::nline
        | Rule::ascii
        | Rule::fun_d
        | Rule::fun_m
        | Rule::asexp
        | Rule::number
        | Rule::fun
        | Rule::letter
        | Rule::WHITESPACE
        | Rule::COMMENT
        | Rule::fun_om
        | Rule::fun_dm
        | Rule::exp
        | Rule::listblock
        | Rule::fnblockl
        | Rule::train
        | Rule::fn_list
        | Rule::texp
        | Rule::terms
        | Rule::block
        | Rule::int
        | Rule::value
        | Rule::ft
        | Rule::string => Err(LocatedError::from(LErrE::FnExpected(err(
            pair.as_str(),
            pair.as_span(),
        )))),
    }
}

fn grab_inner_exprs(pair: Pair) -> (Option<Pair>, Option<Pair>, Option<Pair>) {
    let mut ipair = pair.into_inner();
    (ipair.next(), ipair.next(), ipair.next())
}

fn is_monadic(destruct: &(Option<Pair>, Option<Pair>, Option<Pair>)) -> bool {
    match destruct.2 {
        Some(_) => false,
        _ => true,
    }
}

fn unwrap_dexpr<'a>(
    destruct: (Option<Pair<'a>>, Option<Pair<'a>>, Option<Pair<'a>>),
) -> (PRes, PRes, PRes) {
    (
        destruct.0.ok_or(LocatedError::from(Er)),
        destruct.1.ok_or(LocatedError::from(Er)),
        destruct.2.ok_or(LocatedError::from(Er)),
    )
}

fn unwrap_mexpr<'a>(
    destruct: (Option<Pair<'a>>, Option<Pair<'a>>, Option<Pair<'a>>),
) -> (PRes, PRes) {
    (
        destruct.0.ok_or(LocatedError::from(Er)),
        destruct.1.ok_or(LocatedError::from(Er)),
    )
}

fn delve<'a>(value: &'a str, pair: Pair<'a>) -> BoRes<'a> {
    fn delve_if_present<'b>(value: &'b str, pair: Option<Pair<'b>>) -> BoRes<'b> {
        match pair {
            Some(v) => match v.as_rule() {
                Rule::value | Rule::fnvalue => Ok(v.as_str() == "w"),
                _ => delve(value, v),
            },
            _ => Ok(false),
        }
    }

    pair.into_inner().fold(Ok(false), |r, v| {
        Ok(match v.as_rule() {
            Rule::exp
            | Rule::terms
            | Rule::block
            | Rule::dblock
            | Rule::block_exp
            | Rule::fn_list
            | Rule::listblock => r? || delve(value, v)?,
            Rule::fnvalue | Rule::value => r? || v.as_str() == "w",
            Rule::asexp => {
                let (_, v, _) = grab_inner_exprs(v.clone());
                r? || delve(value, v.ok_or(Er)?)?
            }
            Rule::train | Rule::fun | Rule::dtrain => {
                let (v, w, x) = grab_inner_exprs(v.clone());
                r? || delve_if_present(value, v)?
                    || delve_if_present(value, w)?
                    || delve_if_present(value, x)?
            }
            Rule::cfn => {
                let (v, _, _) = grab_inner_exprs(v.clone());
                r? || delve(value, v.ok_or(Er)?)?
            }
            Rule::EOI
            | Rule::prg
            | Rule::exp_list
            | Rule::op
            | Rule::string
            | Rule::fnblock
            | Rule::term
            | Rule::nline
            | Rule::fnblockl
            | Rule::fnblocko
            | Rule::fnblockm
            | Rule::ascii
            | Rule::fun_d
            | Rule::fun_m
            | Rule::number
            | Rule::texp
            | Rule::letter
            | Rule::WHITESPACE
            | Rule::op_exp
            | Rule::mop
            | Rule::fun_om
            | Rule::fun_dm
            | Rule::COMMENT
            | Rule::cn
            | Rule::int
            | Rule::ft => r?,
        })
    })
}

fn build_ast_from_term(pair: Pair) -> Res {
    debug_println!("term: {}", pair);
    match pair.as_rule() {
        Rule::int => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "_" => (-1, &istr[1..]),
                _ => (1, &istr[..]),
            };
            let integer: i32 = istr.parse()?;
            to_int(pair, sign * integer)
        }
        Rule::ft => {
            let dstr = pair.as_str();
            let (sign, dstr) = match &dstr[..1] {
                "_" => (-1.0, &dstr[1..]),
                _ => (1.0, &dstr[..]),
            };
            let mut flt: f64 = dstr.parse()?;
            if flt != 0.0 {
                // Avoid negative zeroes; only multiply sign by nonzeroes.
                flt *= sign;
            }
            to_float(pair, flt)
        }
        Rule::listblock => {
            let terms: VRes = pair.clone().into_inner().map(build_ast_from_term).collect();
            to_list(pair, terms?)
        }
        Rule::exp => build_ast_from_expr(pair, false),
        Rule::terms => build_ast_from_expr(pair, false),
        Rule::fnblock | Rule::fnblocko => build_ast_from_expr(pair, false),
        Rule::fnblockm => build_ast_from_expr(pair, true),
        Rule::block => build_ast_from_expr(pair, false),
        Rule::string => build_ast_from_expr(pair, false),
        Rule::value => to_val(pair.clone(), pair.as_str()),
        Rule::EOI
        | Rule::prg
        | Rule::op_exp
        | Rule::mop
        | Rule::exp_list
        | Rule::fnblockl
        | Rule::op
        | Rule::term
        | Rule::fun_d
        | Rule::fnvalue
        | Rule::fun_m
        | Rule::nline
        | Rule::ascii
        | Rule::number
        | Rule::letter
        | Rule::train
        | Rule::fn_list
        | Rule::dblock
        | Rule::fun_om
        | Rule::fun_dm
        | Rule::texp
        | Rule::block_exp
        | Rule::cfn
        | Rule::fun
        | Rule::WHITESPACE
        | Rule::COMMENT
        | Rule::asexp
        | Rule::dtrain
        | Rule::cn => Err(LocatedError::from(LErrE::TermExpected(err(
            pair.as_str(),
            pair.as_span(),
        )))),
    }
}

fn build_ast_from_expr(pair: Pair, _monadic: bool) -> Res {
    match pair.as_rule() {
        Rule::exp => build_ast_from_expr(pair.into_inner().next().ok_or(Er)?, false),
        Rule::asexp => {
            let (op_pair, e_pair, _) = grab_inner_exprs(pair.clone());

            let op = NN::new(
                op_pair.clone().ok_or(Er)?.clone(),
                E::VAL(String::from(op_pair.ok_or(Er)?.as_str())),
            );
            to_asexp(
                pair,
                op,
                build_ast_from_expr(e_pair.clone().ok_or(Er)?, false)?,
            )
        }

        Rule::fun | Rule::train => {
            let destruct = grab_inner_exprs(pair.clone());
            match is_monadic(&destruct) {
                true => {
                    let (op, rhs) = unwrap_mexpr(destruct);
                    let op = parse_train(op?, true)?;
                    let rhs = build_ast_from_expr(rhs?, true)?;

                    to_mexp(pair, op, rhs)
                }
                false => {
                    let (lhs, op, rhs) = unwrap_dexpr(destruct);
                    let lhs = build_ast_from_expr(lhs?, false)?;
                    let op = parse_train(op?, false)?;
                    let rhs = build_ast_from_expr(rhs?, false)?;

                    to_dexp(pair, op, lhs, rhs)
                }
            }
        }
        Rule::string => {
            let str = &pair.as_str();
            let str = &str[1..str.len() - 1];
            let str = str.replace("\"\"", "\"");

            to_str(pair, &str)
        }
        Rule::terms => {
            let terms: VRes = pair.clone().into_inner().map(build_ast_from_term).collect();
            let terms = terms?;
            match terms.len() {
                1 => Ok(terms.get(0).ok_or(Er)?.clone()),
                _ => to_term(pair, terms),
            }
        }
        Rule::value => to_val(pair.clone(), pair.as_str()),
        Rule::block => to_block(
            pair.clone(),
            build_ast_from_expr(pair.into_inner().next().ok_or(Er)?, false)?,
        ),
        Rule::fnblock => {
            let ipair = pair.clone().into_inner();
            let m = !delve("w", pair.clone())?;
            let ops: VRes = ipair
                .map(|fun| build_ast_from_expr(fun.clone(), m))
                .collect();

            Ok(NN::new(
                pair,
                match m {
                    true => E::MFBLOCK(ops?),
                    false => E::FBLOCK(ops?),
                },
            ))
        }
        Rule::fnblocko => {
            let mut ipair = pair.clone().into_inner();
            // let m = !delve("w", pair.clone())?;
            let ops = parse_train(ipair.next().ok_or(Er)?, true);
            Ok(NN::new(pair, E::TFBLOCK(ops?)))
        }
        Rule::fnblockm => {
            let mut ipair = pair.clone().into_inner();
            // let m = !delve("w", pair.clone())?;
            let ops = parse_train(ipair.next().ok_or(Er)?, true);
            Ok(NN::new(pair, E::TMFBLOCK(ops?)))
        }
        Rule::EOI
        | Rule::prg
        | Rule::fun_om
        | Rule::fun_dm
        | Rule::cfn
        | Rule::fnvalue
        | Rule::exp_list
        | Rule::op
        | Rule::dblock
        | Rule::term
        | Rule::fnblockl
        | Rule::op_exp
        | Rule::mop
        | Rule::nline
        | Rule::ascii
        | Rule::number
        | Rule::texp
        | Rule::block_exp
        | Rule::fun_d
        | Rule::fun_m
        | Rule::fn_list
        | Rule::dtrain
        | Rule::letter
        | Rule::WHITESPACE
        | Rule::COMMENT
        | Rule::cn
        | Rule::listblock
        | Rule::int
        | Rule::ft => unreachable!(
            "expected an operator str: {}, rule: {:?}",
            pair.as_str(),
            pair.as_rule()
        ),
    }
}

fn to_asexp(pair: Pair, op: NN, e: NN) -> Res {
    Ok(NN::new(
        pair,
        E::ASEXP {
            op: Box::new(op),
            rhs: Box::new(e),
        },
    ))
}

fn to_dblock(p: Pair, op: Vec<NN>, lhs: NN) -> Res {
    Ok(NN::new(
        p,
        E::DBLOCK {
            op,
            lhs: Box::new(lhs),
        },
    ))
}

fn to_ddtrain(p: Pair, op: Vec<NN>, lhs: NN) -> Res {
    Ok(NN::new(
        p,
        E::DDTRAIN {
            op,
            lhs: Box::new(lhs),
        },
    ))
}

fn to_mexp(pair: Pair, op: Vec<NN>, rhs: NN) -> Res {
    Ok(NN::new(
        pair,
        E::MEXP {
            op,
            rhs: Box::new(rhs),
        },
    ))
}

fn to_dexp(pair: Pair, op: Vec<NN>, lhs: NN, rhs: NN) -> Res {
    Ok(NN::new(
        pair,
        E::DEXP {
            op,
            rhs: Box::new(rhs),
            lhs: Box::new(lhs),
        },
    ))
}

fn to_int(p: Pair, i: i32) -> Res {
    Ok(NN::new(p, E::INT(i)))
}
fn to_float(p: Pair, f: f64) -> Res {
    Ok(NN::new(p, E::FT(f)))
}

fn to_list(p: Pair, l: Vec<NN>) -> Res {
    Ok(NN::new(p, E::LIST(l)))
}

fn to_term(p: Pair, terms: Vec<NN>) -> Res {
    Ok(NN::new(p, E::LIST(terms)))
}

fn to_val(p: Pair, val: &str) -> Res {
    Ok(NN::new(p, E::VAL(String::from(val))))
}

fn to_fnval(p: Pair, val: &str) -> Res {
    Ok(NN::new(p, E::FVAL(String::from(val))))
}

fn to_block(p: Pair, val: NN) -> Res {
    Ok(NN::new(p.clone(), E::BL(Box::new(val))))
}

fn to_str(p: Pair, val: &str) -> Res {
    Ok(NN::new(p.clone(), E::ST(String::from(val))))
}

fn to_fblock(p: Pair, list: Vec<NN>) -> Res {
    Ok(NN::new(p, E::FBLOCK(list)))
}

fn to_tfblock(p: Pair, list: Vec<NN>) -> Res {
    Ok(NN::new(p, E::TFBLOCK(list)))
}

fn to_tmfblock(p: Pair, list: Vec<NN>) -> Res {
    Ok(NN::new(p, E::TMFBLOCK(list)))
}

fn to_mfn(p: Pair) -> Res {
    Ok(NN::new(p.clone(), E::MFN(FN::from_string(p.as_str()))))
}

fn to_cn(p: Pair) -> Res {
    Ok(NN::new(p.clone(), E::CN(CN::from_string(p.as_str()))))
}

fn to_dfn(p: Pair) -> Res {
    Ok(NN::new(p.clone(), E::DFN(FN::from_string(p.as_str()))))
}
