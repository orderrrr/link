use l::{
    ast::{CN, E, FN, NN},
    parse::parse,
};

fn parse_ok(input: &str) -> Vec<NN> {
    let p = parse(input);
    assert!(p.is_ok(), "parse failed for '{}': {:?}", input, p.err());
    p.unwrap()
}

#[test]
fn parse_int() {
    let ast = parse_ok("42");
    assert_eq!(ast.len(), 1);
    assert_eq!(ast[0].n, E::INT(42));
}

#[test]
fn parse_negative_int() {
    let ast = parse_ok("-10");
    assert_eq!(ast.len(), 1);
    assert_eq!(ast[0].n, E::INT(-10));
}

#[test]
fn parse_float() {
    let ast = parse_ok("3.14");
    assert_eq!(ast.len(), 1);
    assert_eq!(ast[0].n, E::FT(3.14));
}

#[test]
fn parse_string() {
    let ast = parse_ok("\"hello\"");
    assert_eq!(ast.len(), 1);
    assert_eq!(ast[0].n, E::ST("hello".to_string()));
}

#[test]
fn parse_monadic_apply() {
    // (-| 5) → APPLY { train: [MFN(Minus)], args: [INT(5)] }
    let ast = parse_ok("(-| 5)");
    assert_eq!(ast.len(), 1);
    match &ast[0].n {
        E::APPLY { train, args } => {
            assert_eq!(train.len(), 1);
            assert_eq!(train[0].n, E::MFN(FN::Minus));
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].n, E::INT(5));
        }
        other => panic!("expected APPLY, got {:?}", other),
    }
}

#[test]
fn parse_dyadic_apply() {
    // (+| 1 2) → APPLY { train: [MFN(Plus)], args: [INT(1), INT(2)] }
    let ast = parse_ok("(+| 1 2)");
    assert_eq!(ast.len(), 1);
    match &ast[0].n {
        E::APPLY { train, args } => {
            assert_eq!(train.len(), 1);
            assert_eq!(train[0].n, E::MFN(FN::Plus));
            assert_eq!(args.len(), 2);
            assert_eq!(args[0].n, E::INT(1));
            assert_eq!(args[1].n, E::INT(2));
        }
        other => panic!("expected APPLY, got {:?}", other),
    }
}

#[test]
fn parse_train() {
    // (+/!| 10) → APPLY { train: [MCO{+,/}, MFN(!)], args: [INT(10)] }
    let ast = parse_ok("(+/!| 10)");
    assert_eq!(ast.len(), 1);
    match &ast[0].n {
        E::APPLY { train, args } => {
            assert_eq!(train.len(), 2);
            // First element: +/ (MCO)
            match &train[0].n {
                E::MCO { o, co } => {
                    assert_eq!(o.n, E::MFN(FN::Plus));
                    assert_eq!(co.n, E::CN(CN::Fold));
                }
                other => panic!("expected MCO, got {:?}", other),
            }
            // Second element: ! (MFN)
            assert_eq!(train[1].n, E::MFN(FN::Bang));
            // Arg
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].n, E::INT(10));
        }
        other => panic!("expected APPLY, got {:?}", other),
    }
}

#[test]
fn parse_list_literal() {
    // (1 2 3) → LIST
    let ast = parse_ok("(1 2 3)");
    assert_eq!(ast.len(), 1);
    match &ast[0].n {
        E::LIST(elems) => {
            assert_eq!(elems.len(), 3);
            assert_eq!(elems[0].n, E::INT(1));
            assert_eq!(elems[1].n, E::INT(2));
            assert_eq!(elems[2].n, E::INT(3));
        }
        other => panic!("expected LIST, got {:?}", other),
    }
}

#[test]
fn parse_nested_list() {
    // ((1 2) (3 4)) → LIST of LISTs
    let ast = parse_ok("((1 2) (3 4))");
    assert_eq!(ast.len(), 1);
    match &ast[0].n {
        E::LIST(rows) => {
            assert_eq!(rows.len(), 2);
            match &rows[0].n {
                E::LIST(r) => {
                    assert_eq!(r.len(), 2);
                    assert_eq!(r[0].n, E::INT(1));
                    assert_eq!(r[1].n, E::INT(2));
                }
                other => panic!("expected inner LIST, got {:?}", other),
            }
        }
        other => panic!("expected LIST, got {:?}", other),
    }
}

#[test]
fn parse_lambda() {
    // (λ (x) (+| x 1))
    let ast = parse_ok("(λ (x) (+| x 1))");
    assert_eq!(ast.len(), 1);
    match &ast[0].n {
        E::LAMBDA { params, body } => {
            assert_eq!(params, &vec!["x".to_string()]);
            assert_eq!(body.len(), 1);
        }
        other => panic!("expected LAMBDA, got {:?}", other),
    }
}

#[test]
fn parse_doblock() {
    // (↻ (: x 5) (+| x 3))
    let ast = parse_ok("(↻ (: x 5) (+| x 3))");
    assert_eq!(ast.len(), 1);
    match &ast[0].n {
        E::DOBLOCK(exprs) => {
            assert_eq!(exprs.len(), 2);
            // First: assignment
            match &exprs[0].n {
                E::ASEXP { name, rhs } => {
                    assert_eq!(name, "x");
                    assert_eq!(rhs.n, E::INT(5));
                }
                other => panic!("expected ASEXP, got {:?}", other),
            }
        }
        other => panic!("expected DOBLOCK, got {:?}", other),
    }
}

#[test]
fn parse_assign() {
    // (: x 42)
    let ast = parse_ok("(: x 42)");
    assert_eq!(ast.len(), 1);
    match &ast[0].n {
        E::ASEXP { name, rhs } => {
            assert_eq!(name, "x");
            assert_eq!(rhs.n, E::INT(42));
        }
        other => panic!("expected ASEXP, got {:?}", other),
    }
}

#[test]
fn parse_comment() {
    // 42 with a comment
    let ast = parse_ok("; this is a comment\n42");
    assert_eq!(ast.len(), 1);
    assert_eq!(ast[0].n, E::INT(42));
}
