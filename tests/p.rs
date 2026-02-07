use l::ast::CN;
use l::{
    self,
    ast::{oc, od, om, E, FN, NN},
    parse::parse,
};

fn parse_test_lib(i: &str, ast: Vec<NN>, res: &str) {
    let p = parse(i);
    assert!(p.is_ok());
    assert_eq!(p.clone().unwrap(), ast);
    assert_eq!(format!("{}", p.unwrap()[0]), res)
}

#[test]
fn negative_number_becomes_monadic() {
    parse_test_lib(
        "-10",
        vec![NN::nd(E::MEXP {
            op: vec![NN::nd(om(FN::Minus))],
            rhs: NN::ndb(E::INT(10)),
        })],
        "-10",
    )
}

#[test]
fn negative_number_becomes_negative_when_monadic_called() {
    parse_test_lib(
        "!-10",
        vec![NN::nd(E::MEXP {
            op: NN::ndv(om(FN::Bang)),
            rhs: NN::ndb(E::INT(-10)),
        })],
        "!-10",
    )
}

#[test]
fn negative_number_becomes_negative_when_dyadic_called() {
    parse_test_lib(
        "10 + -10",
        vec![NN::nd(E::DEXP {
            op: NN::ndv(od(FN::Plus)),
            lhs: NN::ndb(E::INT(10)),
            rhs: NN::ndb(E::INT(-10)),
        })],
        "10+-10",
    )
}

#[test]
fn negative_number_becomes_negative_when_monadic_train_called() {
    parse_test_lib(
        "10 |+/!| -10",
        vec![NN::nd(E::DEXP {
            op: vec![
                NN::nd(E::DCO {
                    o: NN::ndb(od(FN::Plus)),
                    co: NN::ndb(oc(CN::Fold)),
                }),
                NN::nd(od(FN::Bang)),
            ],
            lhs: NN::ndb(E::INT(10)),
            rhs: NN::ndb(E::INT(-10)),
        })],
        "10|+/!|-10",
    )
}

#[test]
fn unary_expr() {
    let plus_one = parse("!1");
    assert!(plus_one.is_ok());
    assert_eq!(
        plus_one.clone().unwrap(),
        vec![NN::nd(E::MEXP {
            op: NN::ndv(om(FN::Bang)),
            rhs: NN::ndb(E::INT(1)),
        })]
    );
    assert_eq!(format!("{}", plus_one.unwrap()[0]), "!1");
}

#[test]
fn binary_expr() {
    let sum = parse("1 + 2");
    assert!(sum.is_ok());
    assert_eq!(
        sum.clone().unwrap(),
        NN::ndv(E::DEXP {
            op: NN::ndv(od(FN::Plus)),
            lhs: NN::ndb(E::INT(1)),
            rhs: NN::ndb(E::INT(2))
        })
    );
    assert_eq!(format!("{}", sum.unwrap()[0]), "1+2");
    let minus = parse("1   -  \t  2");
    assert!(minus.is_ok());
    assert_eq!(
        minus.clone().unwrap(),
        NN::ndv(E::DEXP {
            op: NN::ndv(od(FN::Minus)),
            lhs: NN::ndb(E::INT(1)),
            rhs: NN::ndb(E::INT(2))
        })
    );
    assert_eq!(format!("{}", minus.unwrap()[0]), "1-2");
}

#[test]
fn multi_expr() {
    parse_test_lib(
        "x:{(10+a)|+/!|10}",
        vec![NN::nd(E::ASEXP {
            op: NN::ndb(E::VAL(String::from("x"))),
            rhs: NN::ndb(E::MFBLOCK(NN::ndv(E::DEXP {
                op: vec![
                    NN::nd(E::DCO {
                        o: NN::ndb(od(FN::Plus)),
                        co: NN::ndb(oc(CN::Fold)),
                    }),
                    NN::nd(od(FN::Bang)),
                ],
                lhs: NN::ndb(E::BL(NN::ndb(E::DEXP {
                    op: NN::ndv(od(FN::Plus)),
                    lhs: NN::ndb(E::INT(10)),
                    rhs: NN::ndb(E::VAL(String::from("a"))),
                }))),
                rhs: NN::ndb(E::INT(10)),
            }))),
        })],
        "x:{(10+a)|+/!|10}",
    );
    parse_test_lib(
        "10! -10",
        vec![NN::nd(E::DEXP {
            op: NN::ndv(od(FN::Bang)),
            lhs: NN::ndb(E::INT(10)),
            rhs: NN::ndb(E::INT(-10)),
        })],
        "10!-10",
    );
    parse_test_lib(
        "+/10 10",
        vec![NN::nd(E::MEXP {
            op: NN::ndv(E::MCO {
                o: NN::ndb(om(FN::Plus)),
                co: NN::ndb(oc(CN::Fold)),
            }),
            rhs: NN::ndb(E::LIST(vec![NN::nd(E::INT(10)), NN::nd(E::INT(10))])),
        })],
        "+/10 10",
    );
    parse_test_lib(
        "a:x| 10",
        vec![NN::nd(E::ASEXP {
            op: NN::ndb(E::VAL(String::from("a"))),
            rhs: NN::ndb(E::MEXP {
                op: NN::ndv(E::FVAL(String::from("x"))),
                rhs: NN::ndb(E::INT(10)),
            }),
        })],
        "a:x10", // TODO fix printing
    );
}

#[test]
fn list_block() {
    parse_test_lib(
        "[1;2;3]",
        vec![NN::nd(E::LIST(vec![
            NN::nd(E::INT(1)),
            NN::nd(E::INT(2)),
            NN::nd(E::INT(3)),
        ]))],
        "1 2 3",
    );

    parse_test_lib(
        "[1 2 3;2 3 4;3 4 5]",
        vec![NN::nd(E::LIST(vec![
            NN::nd(E::LIST(vec![
                NN::nd(E::INT(1)),
                NN::nd(E::INT(2)),
                NN::nd(E::INT(3)),
            ])),
            NN::nd(E::LIST(vec![
                NN::nd(E::INT(2)),
                NN::nd(E::INT(3)),
                NN::nd(E::INT(4)),
            ])),
            NN::nd(E::LIST(vec![
                NN::nd(E::INT(3)),
                NN::nd(E::INT(4)),
                NN::nd(E::INT(5)),
            ])),
        ]))],
        "1 2 3\n2 3 4\n3 4 5",
    )
}
