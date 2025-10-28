use std::collections::HashMap;

use l::{
    ast::{E, FN, NN},
    byte::{B, I},
    op::{make_op, OP}, get_fnop,
};

#[test]
fn basics() {
    let b = I::fstring("-1");
    let ei = vec![
        OP::CONST(0), 
        OP::MBL(9), 
        OP::MO(get_fnop(FN::Minus)),
        OP::END,
        OP::POP]
        .into_iter()
        .flat_map(make_op)
        .collect();
    assert_eq!(
        B {
            op: ei,
            var: vec![NN {
                n: E::INT(1),
                start: 1,
                end: 2,
            }],
            lookup: HashMap::from([(String::from("a"), 0)]),
            code: HashMap::new(),
        },
        b.unwrap()
    );
}

#[test]
fn closures() {
    let b = I::fstring("{-a}|1");
    let ei = vec![
        OP::CONST(0),
        OP::MBL(20),
        OP::MBL(18),
        OP::CONST(0),
        OP::MBL(18),
        OP::MO(get_fnop(FN::Minus)),
        OP::END,
        OP::END,
        OP::END,
        OP::POP,
    ]
    .into_iter()
    .flat_map(make_op)
    .collect();
    let exp = B {
        op: ei,
        var: vec![NN {
            n: E::INT(1),
            start: 4,
            end: 5,
        }],
        lookup: HashMap::from([(String::from("a"), 0)]),
        code: HashMap::new(),
    };

    assert_eq!(exp, b.unwrap())
}
