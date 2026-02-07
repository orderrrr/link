// Bytecode tests for the new S-expression syntax
// These test the compilation pipeline: source → parse → bytecode

use l::byte::I;

#[test]
fn compile_int() {
    // A bare integer should compile and produce bytecode
    let b = I::fstring("42");
    assert!(b.is_ok(), "failed to compile '42': {:?}", b.err());
    let b = b.unwrap();
    assert!(!b.op.is_empty());
    assert_eq!(b.var.len(), 1); // one constant: 42
}

#[test]
fn compile_monadic() {
    // (-| 1) should compile
    let b = I::fstring("(-| 1)");
    assert!(b.is_ok(), "failed to compile '(-| 1)': {:?}", b.err());
    let b = b.unwrap();
    assert!(!b.op.is_empty());
}

#[test]
fn compile_dyadic() {
    // (+| 2 3) should compile
    let b = I::fstring("(+| 2 3)");
    assert!(b.is_ok(), "failed to compile '(+| 2 3)': {:?}", b.err());
}

#[test]
fn compile_train() {
    // (+/!| 10) should compile
    let b = I::fstring("(+/!| 10)");
    assert!(b.is_ok(), "failed to compile '(+/!| 10)': {:?}", b.err());
}

#[test]
fn compile_list_literal() {
    // (1 2 3) should compile as a list
    let b = I::fstring("(1 2 3)");
    assert!(b.is_ok(), "failed to compile '(1 2 3)': {:?}", b.err());
}
