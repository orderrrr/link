use debug_print::debug_println;
use l::{
    ast::{E, NN},
    byte::I,
    vm::V, parse::pf,
};

fn pe(i: &str) -> NN {
    let byte_code = I::fstring(i);
    let mut vm = V::new(byte_code.unwrap());
    vm.r();
    vm.pop_last().to_owned()
}

fn assert_pop_last(source: &str, node: NN) {
    let byte_code = I::fstring(source);
    debug_println!("byte code: {:?}", byte_code);
    let mut vm = V::new(byte_code.unwrap());
    vm.r();
    assert_eq!(&node, vm.pop_last());
}

#[test]
fn unary() {
    // assert_pop_last("+1", NN::nd(E::INT(1)));
    assert_pop_last("-2", NN::nd(E::INT(-2)));
}

#[test]
fn binary() {
    assert_pop_last("2 + 2;", pf("4"));
    assert_pop_last("1 - 2;", pe("-1"));
}

#[test]
fn lists() {
    assert_pop_last(
        "!4",
        pf("0 1 2 3")
    )
}

#[test]
fn combinator() {
    assert_pop_last("|+/!|10", pf("45"))
}

#[test]
fn function_casting() {
    assert_pop_last("3 5|=:%!.\\!:|4", pf("[1 0 0 1;1 0 0 0]"))
}


#[test]
fn hello_world() {
    assert_pop_last("\"hello world\"", pf("\"hello world\""))
}
