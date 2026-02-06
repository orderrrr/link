use std::{collections::HashMap, fs};

use debug_print::debug_println;
use l::{
    ast::{oc, od, om, CN, E, FN, NN},
    byte::{B, I},
    get_cnop, get_fnop,
    op::{make_op, OP},
    parse::parse,
    pf,
    vm::V,
};

fn parse_test_lib(i: &str, ast: Vec<NN>, res: &str, exp: B, last: NN) {
    let p = parse(i).unwrap();
    assert_eq!(p.clone(), ast);
    assert_eq!(format!("{}", p[0]), res);
    let b = I::fstring(i);
    assert_eq!(exp, b.unwrap());
    assert_pop_last(exp, last);
}

fn assert_pop_last(byte_code: B, node: NN) {
    let mut vm = V::new(byte_code);
    vm.r();
    assert!(vm.error.is_none(), "VM error: {}", vm.error.unwrap());
    assert_eq!(&node, vm.pop_last().unwrap());
}

// TODO: test files use old syntax (%, #, .) not in current grammar
// #[test]
fn euler_001() {
    let ei = vec![
        OP::CONST(0),
        OP::MBL(40),
        OP::MO(get_fnop(FN::Bang)),
        OP::DBL(35),
        OP::DUP(32),
        OP::DBL(25),
        OP::CONST(1),
        OP::DO(get_fnop(FN::Bang)),
        OP::CO(get_cnop(CN::ScanL)),
        OP::END,
        OP::MO(get_fnop(FN::Eq)),
        OP::MO(get_fnop(FN::Max)),
        OP::CO(get_cnop(CN::Fold)),
        OP::END,
        OP::DO(get_fnop(FN::Amp)),
        OP::END,
        OP::MO(get_fnop(FN::Plus)),
        OP::CO(get_cnop(CN::Fold)),
        OP::END,
        OP::POP,
    ]
    .into_iter()
    .flat_map(make_op)
    .collect();

    parse_test_lib(
        &fs::read_to_string("tests/e/001.l").expect("Should have been able to read the file"),
        vec![NN::nd(E::MEXP {
            op: vec![
                NN::nd(E::MCO {
                    o: NN::ndb(om(FN::Plus)),
                    co: NN::ndb(oc(CN::Fold)),
                }),
                NN::nd(E::DBLOCK {
                    op: NN::ndv(E::DFN(FN::Amp)),
                    lhs: NN::ndb(E::MTRAIN(vec![
                        NN::nd(E::MCO {
                            o: NN::ndb(om(FN::Max)),
                            co: NN::ndb(oc(CN::Fold)),
                        }),
                        NN::nd(om(FN::Eq)),
                        NN::nd(E::DDTRAIN {
                            op: vec![NN::nd(E::DCO {
                                o: NN::ndb(od(FN::Bang)),
                                co: NN::ndb(oc(CN::ScanL)),
                            })],
                            lhs: NN::ndb(E::LIST(vec![NN::nd(E::INT(3)), NN::nd(E::INT(5))])),
                        }),
                    ])),
                }),
                NN::nd(om(FN::Bang)),
            ],
            rhs: NN::ndb(E::INT(1000)),
        })],
        "|+/(^/=3 5|%!.\\)|&!|1000", // expected structure
        B {
            op: ei,
            var: vec![
                NN {
                    n: E::INT(1000),
                    start: 4,
                    end: 5,
                },
                NN::nd(E::LIST(vec![NN::nd(E::INT(3)), NN::nd(E::INT(5))])),
            ],
            lookup: HashMap::from([(String::from("a"), 0)]),
            code: HashMap::new(),
        },
        NN::nd(E::INT(233168)),
    )
}

// #[test]
fn euler_001_alt() {
    let ei = vec![
        OP::JMP(40),
        OP::MBL(38),
        OP::GETR,
        OP::MO(get_fnop(FN::Bang)),
        OP::DBL(34),
        OP::DUP(31),
        OP::DBL(24),
        OP::GETL,
        OP::DO(get_fnop(FN::Bang)),
        OP::CO(get_cnop(CN::ScanL)),
        OP::END,
        OP::MO(get_fnop(FN::Eq)),
        OP::MO(get_fnop(FN::Max)),
        OP::CO(get_cnop(CN::Fold)),
        OP::END,
        OP::DO(get_fnop(FN::Amp)),
        OP::END,
        OP::MO(get_fnop(FN::Plus)),
        OP::CO(get_cnop(CN::Fold)),
        OP::END,
        OP::END,
        OP::CONST(0),
        OP::CONST(1),
        OP::DBL(53),
        OP::JMP(3),
        OP::END,
        OP::POP,
    ]
    .into_iter()
    .flat_map(make_op)
    .collect();

    parse_test_lib(
        &fs::read_to_string("tests/e/001-alt.l").expect("Should have been able to read the file"),
        // vec![],
        vec![
            NN::nd(E::ASEXP {
                op: NN::ndb(E::VAL(String::from("fn"))),
                rhs: NN::ndb(E::TMFBLOCK(vec![
                    NN::nd(E::MCO {
                        o: NN::ndb(E::MFN(FN::Plus)),
                        co: NN::ndb(E::CN(CN::Fold)),
                    }),
                    NN::nd(E::DBLOCK {
                        op: NN::ndv(E::DFN(FN::Amp)),
                        lhs: NN::ndb(E::MTRAIN(vec![
                            NN::nd(E::MCO {
                                o: NN::ndb(om(FN::Max)),
                                co: NN::ndb(oc(CN::Fold)),
                            }),
                            NN::nd(om(FN::Eq)),
                            NN::nd(E::DDTRAIN {
                                op: vec![NN::nd(E::DCO {
                                    o: NN::ndb(od(FN::Bang)),
                                    co: NN::ndb(oc(CN::ScanL)),
                                })],
                                lhs: NN::ndb(E::VAL(String::from("w"))),
                            }),
                        ])),
                    }),
                    NN::nd(om(FN::Bang)),
                ])),
            }),
            NN::nd(E::DEXP {
                op: NN::ndv(E::FVAL(String::from("fn"))),
                lhs: NN::ndb(E::LIST(vec![NN::nd(E::INT(3)), NN::nd(E::INT(5))])),
                rhs: NN::ndb(E::INT(1000)),
            }),
        ],
        "fn:{+/(^/=w|%!.\\)|&!|}", // expected structure
        B {
            op: ei,

            // [NN { n: INT(1000), start: 33, end: 37 }, NN { n: LIST([NN { n: INT(3), start: 24, end: 25 }, NN { n: INT(5), start: 26, end: 27 }]), start: 24, end: 27 }]
            var: vec![
                NN::nd(E::INT(1000)),
                NN::nd(E::LIST(vec![NN::nd(E::INT(3)), NN::nd(E::INT(5))])),
            ],
            lookup: HashMap::from([
                (String::from("w"), 1),
                (String::from("fn"), 3),
                (String::from("a"), 0),
            ]),
            code: HashMap::new(),
        },
        NN::nd(E::INT(233168)),
    )
}

fn assert_pop_last_st(source: &str, node: NN) {
    let byte_code = I::fstring(source);
    debug_println!("byte code: {:?}", byte_code);
    let mut vm = V::new(byte_code.unwrap());
    vm.r();
    assert!(vm.error.is_none(), "VM error: {}", vm.error.unwrap());
    assert_eq!(&node, vm.pop_last().unwrap());
}

// #[test]
fn euler_003() {
    assert_pop_last_st(
        &fs::read_to_string("tests/e/003.li").expect("Should have been able to read the file"),
        pf("4"),
    );
}
