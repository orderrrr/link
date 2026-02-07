use l::{
    ast::{E, NN},
    byte::I,
    vm::V,
};

fn assert_pop_last(source: &str, node: NN) {
    let byte_code = I::fstring(source);
    let mut vm = V::new(byte_code.unwrap());
    vm.r();
    assert!(vm.error.is_none(), "VM error: {}", vm.error.unwrap());
    assert_eq!(&node, vm.pop_last().unwrap());
}

#[test]
fn unary() {
    // (-| 2) → -2
    assert_pop_last("(-| 2)", NN::nd(E::INT(-2)));
}

#[test]
fn binary() {
    // (+| 2 2) → 4
    assert_pop_last("(+| 2 2)", NN::nd(E::INT(4)));
    // (-| 1 2) → -1
    assert_pop_last("(-| 1 2)", NN::nd(E::INT(-1)));
}

#[test]
fn lists() {
    // (!| 4) → 0 1 2 3
    assert_pop_last(
        "(!| 4)",
        NN::nd(E::LIST(vec![
            NN::nd(E::INT(0)),
            NN::nd(E::INT(1)),
            NN::nd(E::INT(2)),
            NN::nd(E::INT(3)),
        ])),
    );
}

#[test]
fn combinator() {
    // (+/| (!| 10)) → sum of 0..9 = 45
    assert_pop_last("(+/| (!| 10))", NN::nd(E::INT(45)));
}

#[test]
fn hello_world() {
    assert_pop_last("\"hello world\"", NN::nd(E::ST("hello world".to_string())));
}

#[test]
fn list_literal() {
    // (1 2 3) → list [1, 2, 3]
    assert_pop_last(
        "(1 2 3)",
        NN::nd(E::LIST(vec![
            NN::nd(E::INT(1)),
            NN::nd(E::INT(2)),
            NN::nd(E::INT(3)),
        ])),
    );
}

#[test]
fn doblock() {
    // (↻ (+| 1 2) (+| 3 4)) → 7 (returns last)
    assert_pop_last("(↻ (+| 1 2) (+| 3 4))", NN::nd(E::INT(7)));
}

#[test]
fn repl_var_persistence() {
    // Simulate REPL: define a variable on one line, use it on the next.
    // Line 1: (: x 5)
    let b1 = I::fstring("(: x 5)").unwrap();
    let mut vm1 = V::new(b1);
    vm1.r();
    assert!(vm1.error.is_none(), "VM error: {}", vm1.error.unwrap());
    let env = vm1.env();

    // Line 2: x  — should resolve to 5
    let b2 = I::fstring_with_env("x", env.clone()).unwrap();
    let mut vm2 = V::new(b2);
    vm2.r();
    assert!(vm2.error.is_none(), "VM error: {}", vm2.error.unwrap());
    assert_eq!(&NN::nd(E::INT(5)), vm2.pop_last().unwrap());

    // Line 3: define another var using the first
    let env2 = vm2.env();
    let b3 = I::fstring_with_env("(: y (↻ x))", env2).unwrap();
    let mut vm3 = V::new(b3);
    vm3.r();
    assert!(vm3.error.is_none(), "VM error: {}", vm3.error.unwrap());

    // Line 4: retrieve y — should be 5
    let env3 = vm3.env();
    let b4 = I::fstring_with_env("y", env3).unwrap();
    let mut vm4 = V::new(b4);
    vm4.r();
    assert!(vm4.error.is_none(), "VM error: {}", vm4.error.unwrap());
    assert_eq!(&NN::nd(E::INT(5)), vm4.pop_last().unwrap());
}

#[test]
fn lambda_monadic_in_train() {
    // (: test (λ (a) (!| a))) — define range function as a user fn
    // Then: (test| 4) — call it monadically in a train
    // Should produce 0 1 2 3 (same as (!| 4))
    let b1 = I::fstring("(: test (λ (a) (!| a)))").unwrap();
    let mut vm1 = V::new(b1);
    vm1.r();
    assert!(
        vm1.error.is_none(),
        "VM error on define: {}",
        vm1.error.unwrap()
    );
    let env = vm1.env();

    let b2 = I::fstring_with_env("(test| 4)", env).unwrap();
    let mut vm2 = V::new(b2);
    vm2.r();
    assert!(
        vm2.error.is_none(),
        "VM error on call: {}",
        vm2.error.unwrap()
    );
    assert_eq!(
        &NN::nd(E::LIST(vec![
            NN::nd(E::INT(0)),
            NN::nd(E::INT(1)),
            NN::nd(E::INT(2)),
            NN::nd(E::INT(3)),
        ])),
        vm2.pop_last().unwrap()
    );
}

#[test]
fn lambda_in_dyadic_train() {
    // (: test (λ (a) (!| a))) — range function
    // (ρ test| (3 2) 6) should equal (ρ!:| (3 2) 6) — reshape range(6) to 3x2
    let b1 = I::fstring("(: test (λ (a) (!| a)))").unwrap();
    let mut vm1 = V::new(b1);
    vm1.r();
    assert!(
        vm1.error.is_none(),
        "VM error on define: {}",
        vm1.error.unwrap()
    );
    let env = vm1.env();

    // First verify (ρ!:| (3 2) 6) works
    let b_builtin = I::fstring_with_env("(ρ!:| (3 2) 6)", env.clone()).unwrap();
    let mut vm_builtin = V::new(b_builtin);
    vm_builtin.r();
    assert!(
        vm_builtin.error.is_none(),
        "VM error on builtin: {}",
        vm_builtin.error.unwrap()
    );
    let expected = vm_builtin.pop_last().unwrap().clone();

    // Now verify (ρ test| (3 2) 6) produces the same result
    let env2 = vm1.env();
    let b2 = I::fstring_with_env("(ρ test| (3 2) 6)", env2).unwrap();
    let mut vm2 = V::new(b2);
    vm2.r();
    assert!(
        vm2.error.is_none(),
        "VM error on user fn train: {}",
        vm2.error.unwrap()
    );
    assert_eq!(&expected, vm2.pop_last().unwrap());
}
