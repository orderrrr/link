// use l::p;

use std::fs;

use l::{byte::I, vm::V};

fn main() {

    let input = fs::read_to_string("tests/o/001.l").expect("Should have been able to read the file");
    let parse = I::fstring(&input);
    let mut vm = V::new(parse.unwrap());
    vm.r();

    println!("{:#?}", vm.pop_last());
}
