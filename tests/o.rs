use l::{
    ast::FN,
    get_fnop,
    op::{make_op, OP},
};

#[test]
fn make_op_constant() {
    assert_eq!(vec![0x01, 255, 254], make_op(OP::CONST(65534)));
}

#[test]
fn make_op_pop() {
    assert_eq!(vec![0x02], make_op(OP::POP));
}

#[test]
fn make_op_add() {
    assert_eq!(vec![12, 1], make_op(OP::MO(get_fnop(FN::Plus))));
}
