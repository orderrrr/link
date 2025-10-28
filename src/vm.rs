use std::cmp;

use debug_print::debug_println;
use rayon::prelude::*;

use crate::{
    ast::{CN, E, FN, NN},
    byte::B,
    byte_to_fn,
    op::{byte_to_op, u8_to_u, OP},
    op_to_co,
};

const STACK_SIZE: usize = 512;

#[derive(Debug, Copy, Clone)]
pub enum BL {
    JMP,
    DUP,
    DBL,
    MBL,
}

#[derive(Debug, Copy, Clone)]
pub struct C {
    ip: usize,
    t: BL,
}

impl C {
    pub fn new(ip: usize, t: BL) -> C {
        C { ip, t }
    }
}

pub struct V {
    b: B,

    s: [NN; STACK_SIZE],
    ptr: usize,

    context: [C; STACK_SIZE],
    cptr: usize,
}

impl V {
    pub fn new(b: B) -> Self {
        Self {
            b,
            s: unsafe { std::mem::zeroed() },
            ptr: 0,
            context: unsafe { std::mem::zeroed() },
            cptr: 0,
        }
    }

    pub fn r(&mut self) {
        let mut ip = 0;
        while ip < self.b.op.len() {
            let iaddr = ip;
            ip += 1;
            let op = byte_to_op(self.b.op[iaddr]).unwrap();

            debug_println!("self.b.op is {:?}", self.b.op);
            debug_println!("next op, ip is: {:?} {}", op, ip);

            match op {
                OP::CONST(_) => {
                    println!("\n\n-------- CONST --------");
                    let const_idx = u8_to_u(self.b.op[ip], self.b.op[ip + 1]);
                    ip += 2;
                    self.push(self.b.var[const_idx].clone());
                    debug_println!("pushed: {}", self.s[self.ptr - 1]);
                }
                OP::POP => {
                    println!("\n\n-------- POP --------");
                    let prev_op = byte_to_op(self.b.op[ip - 4]);

                    debug_println!("prev op: {:?}", prev_op);

                    self.pop();
                }
                OP::JMP(_) => {
                    println!("\n\n-------- JMP --------");

                    let us = self.get_usize(iaddr);
                    self.cpush(C::new(us, BL::JMP));

                    let i = self.get_usize(iaddr);
                    ip = i as usize;
                }
                OP::DUP(_) => {
                    println!("\n\n-------- DUP --------");

                    let c = self.cget();

                    let us = self.get_usize(iaddr);
                    self.cpush(C::new(us, BL::DUP));
                    println!("cget: {:?}", self.cget());

                    match c.t {
                        BL::DBL => {
                            self.ddup();
                            self.ddup();
                        }
                        _ => self.dup(),
                    }

                    println!("-------- RESULT");
                    println!("ptr: {}", self.ptr);
                    println!("top of stack: {}", self.s[self.ptr - 1]);
                    ip += 2;
                }
                OP::DBL(_) => {
                    println!("\n\n-------- DBL --------");

                    self.ddup();
                    self.ddup();

                    let us = self.get_usize(iaddr);
                    self.cpush(C::new(us, BL::DBL));
                    println!("cget: {:?}", self.cget());

                    println!("-------- RESULT");
                    println!("cget: {:?}", self.cget());
                    println!("ptr: {}", self.ptr);
                    println!("top of stack: {}", self.s[self.ptr - 1]);
                    ip += 2;
                }
                OP::MBL(_) => {
                    println!("\n\n-------- MBL --------");

                    self.dup();

                    let us = self.get_usize(iaddr);
                    self.cpush(C::new(us, BL::MBL));
                    println!("-------- RESULT");
                    println!("cget: {:?}", self.cget());
                    println!("ptr: {}", self.ptr);
                    println!("top of stack: {}", self.s[self.ptr - 1]);

                    ip += 2;
                }
                OP::MO(_) => {
                    let mo = byte_to_fn(self.b.op[ip]);
                    println!("\n\n-------- MO {} --------", mo);

                    println!("ptr: {}", self.ptr);
                    println!("top of stack: {}", self.s[self.ptr - 1]);

                    // TODO - fix messy code
                    match self.cget().t {
                        BL::DBL => self.ptr -= 1,
                        _ => (),
                    }

                    println!("ptr: {}", self.ptr);
                    println!("top of stack: {}", self.s[self.ptr - 1]);

                    let co = byte_to_op(self.b.op[ip + 1]).unwrap();
                    let co = op_to_co(co, self.b.op[ip + 2]);

                    self.cmo(co, mo, ip);

                    println!("-------- RESULT");
                    println!("ptr: {}", self.ptr);
                    println!("top of stack: {}", self.s[self.ptr - 1]);

                    match co {
                        Some(_) => ip += 3,
                        None => ip += 1,
                    }

                    // TODO - fix messy code
                    match self.cget().t {
                        BL::DBL => self.ptr += 1,
                        _ => (),
                    }
                }
                OP::DO(_) => {
                    let dfn = byte_to_fn(self.b.op[ip]);
                    println!("\n\n-------- DO {} --------", dfn);

                    let co = byte_to_op(self.b.op[ip + 1]).unwrap();
                    let co = op_to_co(co, self.b.op[ip + 2]);

                    self.cdo(co, dfn, ip);

                    self.ptr += 1;

                    match co {
                        Some(_) => ip += 3,
                        None => ip += 1,
                    }

                    println!("-------- RESULT");
                    println!("ptr: {}", self.ptr);
                    println!("top of stack: {}", self.s[self.ptr - 1]);
                }
                OP::CLVAR => {
                    println!("\n\n-------- CLVAR --------");
                    println!("pop is {}", self.pop());
                }
                OP::END => {
                    println!("\n\n-------- END --------");

                    let c = self.cpop();

                    // TODO - delete or uncomment
                    match c.t {
                        BL::DBL => {
                            println!("\n\n-------- DBLEND --------");
                            self.pop();
                            let res = self.pop();
                            println!("HERE: {}", self.pop());
                            println!("RES HERE: {}", res);

                            let c = self.cget();
                            match c.t {
                                BL::DBL => {
                                    let rhs = self.pop();
                                    self.push(res);
                                    self.push(rhs);
                                }
                                _ => {
                                    self.push(res);
                                }
                            }
                        }
                        BL::MBL => {
                            println!("\n\n-------- MBLEND --------");
                            let res = self.pop();
                            println!("HERE: {}", self.pop());
                            println!("RES HERE: {}", res);
                            match c.t {
                                BL::DBL => {
                                    let rhs = self.pop();
                                    self.push(res);
                                    self.push(rhs);
                                }
                                _ => {
                                    self.push(res);
                                }
                            }
                        }
                        _ => (),
                    }

                    println!("-------- RESULT");
                    println!("cptr = {}", self.cptr);
                    println!("cip = {}", c.ip);
                    println!("ptr: {}", self.ptr);
                    println!("top of stack: {}", self.s[self.ptr - 1]);
                    ip = c.ip;
                }
                OP::GETL => {
                    println!("\n\n-------- GETL --------");

                    debug_println!("current lhs: {}", self.s[self.ptr - 1]);
                    debug_println!("current rhs: {}", self.s[self.ptr - 2]);
                    self.push(
                        self.b.var[self.b.lookup.get("w").unwrap().to_owned() as usize].clone(),
                    );
                }
                OP::GETR => {
                    println!("\n\n-------- GETR --------");

                    debug_println!("current lhs: {}", self.s[self.ptr - 1]);
                    debug_println!("current rhs: {}", self.s[self.ptr - 2]);
                    self.push(
                        self.b.var[self.b.lookup.get("a").unwrap().to_owned() as usize].clone(),
                    );
                    // self.ptr -= 1;
                    // debug_println!("{:?}", self.s[self.ptr]);
                    // let rhs = self.pop();
                    // self.rep(rhs);
                }
                _ => panic!("unimplemented instruction: {:?}", op),
            }
        }
    }

    pub fn cmo(&mut self, co: Option<CN>, fun: FN, _ip: usize) {
        match co {
            None => {
                let rhs = self.pop();
                debug_println!("cmo: rhs: {}", rhs);
                let (fun, _) = self.get_fun(fun, _ip);
                self.push(fun(&rhs));
            }
            Some(CN::Fold) => {
                let rhs = self.pop();
                debug_println!("cmo: rhs: {}", rhs);
                let (_, fun) = self.get_fun(fun, _ip);
                match rhs.n {
                    E::LIST(l) => self.push(l.into_iter().reduce(|r, a| fun(&r, &a)).unwrap()),
                    _ => panic!("Unknown instruction"),
                };
            }
            _ => panic!("unknown instruction: {:?}", co),
        }
    }

    pub fn cdo(&mut self, co: Option<CN>, fun: FN, _ip: usize) {
        match co {
            None => {
                debug_println!("cdo None");
                let lhs = self.pop();
                let rhs = self.pop();
                debug_println!("cdo lhs: {}", lhs);
                debug_println!("cdo rhs: {}", rhs);

                let (_, fun) = self.get_fun(fun, _ip);
                self.push(fun(&lhs, &rhs));
                // match rhs.n {
                //     E::INT(_) => self.push(func.1(lhs, rhs)),
                //     _ => panic!("Unknown instruction"),
                // }
            }
            Some(CN::ScanL) => {
                debug_println!("cdo ScanL");
                let lhs = self.pop();
                let rhs = self.pop();
                debug_println!("cdo rhs: {}", rhs);
                debug_println!("cdo lhs: {}", lhs);

                let (_, fun) = self.get_fun(fun, _ip);

                match lhs.n {
                    E::LIST(l) => self.push(NN::nd(E::LIST(
                        l.into_iter()
                            .map(|w| match rhs.n.clone() {
                                E::LIST(r) => NN::nd(E::LIST(
                                    r.into_iter().map(|a| fun(&w.clone(), &a)).collect(),
                                )),
                                E::INT(_) => fun(&w, &rhs.clone()),
                                _ => panic!("Unknown instruction"),
                            })
                            .collect(),
                    ))),
                    _ => panic!("Unknown instruction"),
                }
            }
            _ => panic!("unknown instruction"),
        }
    }

    // TODO - do the operator, monadic and type matching inside global do function
    pub fn get_fun(&mut self, fun: FN, _ip: usize) -> (fn(&NN) -> NN, fn(&NN, &NN) -> NN) {
        match fun {
            FN::Bang => (mo_bang, do_temp),
            FN::Eq => (mo_eq, do_temp),
            FN::Div => (mo_temp, do_temp),
            FN::Max => (mo_temp, do_max),
            FN::Min => (mo_min, do_min),
            FN::Amp => (mo_temp, do_amp),
            FN::Plus => (mo_temp, do_plus),
            FN::Minus => (mo_minus, do_minus),
            FN::Mult => (mo_temp, do_temp),
        }
    }

    pub fn get_usize(&mut self, ip: usize) -> usize {
        return u8_to_u(self.b.op[ip + 1], self.b.op[ip + 2]);
    }

    pub fn push(&mut self, node: NN) {
        self.s[self.ptr] = node;
        self.ptr += 1;
    }

    pub fn pop(&mut self) -> NN {
        // ignoring the potential of stack underflow
        // cloning rather than mem::replace for easier testing
        let node = self.s[self.ptr - 1].clone();
        self.ptr -= 1;
        node
    }

    pub fn get(&mut self) -> NN {
        self.s[self.ptr - 1].clone()
    }

    pub fn cpush(&mut self, node: C) {
        self.context[self.cptr] = node;
        self.cptr += 1;
    }

    pub fn cget(&mut self) -> C {
        match self.cptr {
            1.. => self.context[self.cptr - 1],
            _ => C { ip: 0, t: BL::DUP },
        }
    }

    pub fn cpop(&mut self) -> C {
        // ignoring the potential of stack underflow
        // cloning rather than mem::replace for easier testing
        let node = self.context[self.cptr - 1].clone();
        self.cptr -= 1;
        node
    }

    pub fn dup(&mut self) {
        println!("!!!!!! DUP !!!!!!");
        let node = self.s[self.ptr - 1].clone();
        println!("NODE IS: {}", node);
        self.push(node);
    }

    pub fn ddup(&mut self) {
        println!("!!!!!! DDUP !!!!!!");
        let node = self.s[self.ptr - 2].clone();
        println!("NODE IS: {}", node);
        self.push(node);
    }

    pub fn pop_last(&self) -> &NN {
        // the stack pointer points to the next "free" space,
        // which also holds the most recently popped element
        &self.s[self.ptr]
    }
}

pub fn mo_temp(_rhs: &NN) -> NN {
    panic!("Unknown instruction")
}

pub fn mo_bang(rhs: &NN) -> NN {
    match rhs.n {
        E::INT(i) => NN::nd(E::LIST((0..i).map(|el| NN::nd(E::INT(el))).collect())),
        _ => panic!("Unknown instruction"),
    }
}

pub fn mo_minus(rhs: &NN) -> NN {
    match rhs.n {
        E::INT(i) => NN::nd(E::INT(-i)),
        _ => panic!("Unknown instruction"),
    }
}

pub fn mo_eq(rhs: &NN) -> NN {
    match rhs.clone().n {
        E::INT(i) => NN::nd(E::BOOL(i == 0)),
        E::LIST(l) => NN::nd(E::LIST(
            l.into_iter()
                .map(|a| match a.n {
                    E::INT(i) => NN::nd(E::BOOL(i == 0)),
                    E::LIST(_) => mo_eq(&a),
                    _ => panic!("Unknown type"),
                })
                .collect(),
        )),
        _ => panic!("Unknown instruction"),
    }
}

pub fn mo_min(rhs: &NN) -> NN {
    match rhs.clone().n {
        E::FT(i) => NN::nd(E::INT(i.floor() as i32)),
        _ => panic!("Unknown instruction: {}", rhs),
    }
}

pub fn bool_to_int(b: bool) -> i32 {
    match b {
        true => 1,
        false => 0,
    }
}

pub fn do_conversion(lhs: &NN, rhs: &NN, do_target: fn(&NN, &NN) -> NN) -> Option<NN> {
    match (lhs.clone().n, rhs.clone().n) {
        (E::INT(_), E::INT(_)) => Some(do_target(lhs, rhs)),
        (E::BOOL(_), E::BOOL(_)) => Some(do_target(lhs, rhs)),
        (E::LIST(l), E::LIST(r)) => match l.len() == r.len() {
            true => Some(NN::nd(E::LIST(
                l.par_iter().zip(r).map(|(l, r)| do_target(l, &r)).collect(),
            ))),
            false => panic!("Unknown"),
        },
        _ => None,
    }
}

pub fn do_temp(_lhs: &NN, _rhs: &NN) -> NN {
    panic!("Unknown instruction")
}

pub fn do_mathmod(lhs: &NN, rhs: &NN) -> NN {
    match lhs.n {
        E::INT(w) => match rhs.n {
            E::INT(a) => NN::nd(E::INT(a % w)),
            _ => panic!("Unknown instruction"),
        },
        _ => match do_conversion(lhs, rhs, do_mathmod) {
            Some(n) => n,
            None => panic!("Unknown type"),
        },
    }
}

pub fn do_mathdiv(lhs: &NN, rhs: &NN) -> NN {
    println!("math div, lhs: {}; rhs: {}", lhs, rhs);
    match (lhs.clone().n, rhs.clone().n) {
        (E::INT(w), E::INT(a)) => NN::nd(E::FT((a as f64) / (w as f64))),
        _ => match do_conversion(lhs, rhs, do_mathmod) {
            Some(n) => n,
            None => panic!("Unknown type"),
        },
    }
}

pub fn do_max(lhs: &NN, rhs: &NN) -> NN {
    match (lhs.n.clone(), rhs.n.clone()) {
        (E::INT(w), E::INT(a)) => NN::nd(E::INT(cmp::max(a, w))),
        (E::BOOL(w), E::BOOL(a)) => NN::nd(E::BOOL(w || a)),
        _ => match do_conversion(lhs, rhs, do_max) {
            Some(n) => n,
            None => panic!("Unknown type"),
        },
    }
}

pub fn do_min(lhs: &NN, rhs: &NN) -> NN {
    match (lhs.n.clone(), rhs.n.clone()) {
        (E::INT(w), E::INT(a)) => NN::nd(E::INT(cmp::min(a, w))),
        _ => match do_conversion(lhs, rhs, do_max) {
            Some(n) => n,
            None => panic!("Unknown type"),
        },
    }
}

pub fn do_amp(lhs: &NN, rhs: &NN) -> NN {
    match (lhs.clone().n, rhs.clone().n) {
        (E::LIST(l), E::LIST(a)) => NN::nd(E::LIST(l.into_iter().enumerate().fold(
            Vec::new(),
            |mut r: Vec<NN>, (i, w)| match w.n {
                E::BOOL(true) => {
                    r.push(a[i].clone());
                    r
                }
                _ => r,
            },
        ))),
        _ => panic!("Unknown instruction"),
    }
}

pub fn do_plus(lhs: &NN, rhs: &NN) -> NN {
    match (lhs.clone().n, rhs.clone().n) {
        (E::INT(w), E::INT(a)) => NN::nd(E::INT(w + a)),
        (E::BOOL(w), E::BOOL(a)) => NN::nd(E::INT(bool_to_int(w) + bool_to_int(a))),
        _ => match do_conversion(lhs, rhs, do_plus) {
            Some(n) => n,
            None => panic!("Unknown type"),
        },
    }
}

pub fn do_minus(lhs: &NN, rhs: &NN) -> NN {
    match lhs.n {
        E::INT(w) => match rhs.n {
            E::INT(a) => NN::nd(E::INT(w - a)),
            _ => panic!("Unknown instruction"),
        },
        _ => match do_conversion(lhs, rhs, do_minus) {
            Some(n) => n,
            None => panic!("Unknown type"),
        },
    }
}
