use std::cmp;
use std::fmt;

use debug_print::debug_println;
use rayon::prelude::*;

use crate::{
    ast::{CN, E, FN, NN},
    byte::{Env, B},
    byte_to_fn,
    op::{byte_to_op, u8_to_u, OP},
    op_to_co,
};

const STACK_SIZE: usize = 512;

// ---------------------------------------------------------------------------
// VM error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct VMError {
    pub msg: String,
}

impl VMError {
    pub fn new(msg: impl Into<String>) -> Self {
        VMError { msg: msg.into() }
    }
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime error: {}", self.msg)
    }
}

/// Shorthand for a type name we can show to users.
fn type_name(n: &NN) -> &'static str {
    match &n.n {
        E::INT(_) => "int",
        E::FT(_) => "float",
        E::BOOL(_) => "bool",
        E::ST(_) => "string",
        E::LIST(_) => "list",
        E::VAL(_) => "value",
        E::UFNV { .. } => "fn",
        _ => "node",
    }
}

pub type VmRes = Result<NN, VMError>;

type MonadicFn = fn(&NN) -> VmRes;
type DyadicFn = fn(&NN, &NN) -> VmRes;

// ---------------------------------------------------------------------------
// Context stack
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// VM
// ---------------------------------------------------------------------------

pub struct V {
    b: B,

    s: Vec<NN>,
    last_popped: Option<NN>,
    pub error: Option<VMError>,

    context: [C; STACK_SIZE],
    cptr: usize,
}

impl V {
    pub fn new(b: B) -> Self {
        Self {
            b,
            s: Vec::with_capacity(STACK_SIZE),
            last_popped: None,
            error: None,
            context: unsafe { std::mem::zeroed() },
            cptr: 0,
        }
    }

    pub fn r(&mut self) {
        if let Err(e) = self.run() {
            self.error = Some(e);
        }
    }

    /// Extract the environment (variable pool + lookup table) after execution,
    /// so it can be carried into the next REPL iteration.
    pub fn env(&self) -> Env {
        Env {
            var: self.b.var.clone(),
            lookup: self.b.lookup.clone(),
        }
    }

    fn run(&mut self) -> Result<(), VMError> {
        let mut ip = 0;
        while ip < self.b.op.len() {
            let iaddr = ip;
            ip += 1;
            let op = byte_to_op(self.b.op[iaddr]).unwrap();

            debug_println!("self.b.op is {:?}", self.b.op);
            debug_println!("next op, ip is: {:?} {}", op, ip);

            match op {
                OP::CONST(_) => {
                    debug_println!("\n\n-------- CONST --------");
                    let const_idx = u8_to_u(self.b.op[ip], self.b.op[ip + 1]);
                    ip += 2;
                    self.push(self.b.var[const_idx].clone());
                    debug_println!("pushed: {}", self.s.last().unwrap());
                }
                OP::POP => {
                    debug_println!("\n\n-------- POP --------");
                    self.pop();
                }
                OP::JMP(_) => {
                    debug_println!("\n\n-------- JMP --------");

                    let us = self.get_usize(iaddr);
                    self.cpush(C::new(us, BL::JMP));

                    let i = self.get_usize(iaddr);
                    ip = i as usize;
                }
                OP::DUP(_) => {
                    debug_println!("\n\n-------- DUP --------");

                    let c = self.cget();

                    let us = self.get_usize(iaddr);
                    self.cpush(C::new(us, BL::DUP));

                    match c.t {
                        BL::DBL => {
                            self.ddup();
                            self.ddup();
                        }
                        _ => self.dup(),
                    }

                    ip += 2;
                }
                OP::DBL(_) => {
                    debug_println!("\n\n-------- DBL --------");

                    self.ddup();
                    self.ddup();

                    let us = self.get_usize(iaddr);
                    self.cpush(C::new(us, BL::DBL));

                    ip += 2;
                }
                OP::MBL(_) => {
                    debug_println!("\n\n-------- MBL --------");

                    self.dup();

                    let us = self.get_usize(iaddr);
                    self.cpush(C::new(us, BL::MBL));

                    ip += 2;
                }
                OP::MO(_) => {
                    let mo = byte_to_fn(self.b.op[ip]);
                    debug_println!("\n\n-------- MO {} --------", mo);

                    // In a DBL context, temporarily hide the extra duplicated value
                    let dbl_stashed = match self.cget().t {
                        BL::DBL => Some(self.s.pop().expect("stack underflow")),
                        _ => None,
                    };

                    let co = byte_to_op(self.b.op[ip + 1]).unwrap();
                    let co = op_to_co(co, self.b.op[ip + 2]);

                    self.cmo(co, mo, ip)?;

                    match co {
                        Some(_) => ip += 3,
                        None => ip += 1,
                    }

                    // Restore the stashed value after the monadic op
                    if let Some(stashed) = dbl_stashed {
                        self.s.push(stashed);
                    }
                }
                OP::DO(_) => {
                    let dfn = byte_to_fn(self.b.op[ip]);
                    debug_println!("\n\n-------- DO {} --------", dfn);

                    let co = byte_to_op(self.b.op[ip + 1]).unwrap();
                    let co = op_to_co(co, self.b.op[ip + 2]);

                    self.cdo(co, dfn, ip)?;

                    self.dup();

                    match co {
                        Some(_) => ip += 3,
                        None => ip += 1,
                    }
                }
                OP::CLVAR => {
                    debug_println!("\n\n-------- CLVAR --------");
                    self.pop();
                }
                OP::END => {
                    debug_println!("\n\n-------- END --------");

                    let c = self.cpop();

                    match c.t {
                        BL::DBL => {
                            debug_println!("\n\n-------- DBLEND --------");
                            self.pop();
                            let res = self.pop();
                            self.pop();

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
                            debug_println!("\n\n-------- MBLEND --------");
                            let res = self.pop();
                            self.pop();
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

                    ip = c.ip;
                }
                OP::GETL => {
                    debug_println!("\n\n-------- GETL --------");
                    self.push(
                        self.b.var[self.b.lookup.get("w").unwrap().to_owned() as usize].clone(),
                    );
                }
                OP::GETR => {
                    debug_println!("\n\n-------- GETR --------");
                    self.push(
                        self.b.var[self.b.lookup.get("a").unwrap().to_owned() as usize].clone(),
                    );
                }
                OP::STORE(_) => {
                    debug_println!("\n\n-------- STORE --------");
                    let name_idx = u8_to_u(self.b.op[ip], self.b.op[ip + 1]);
                    ip += 2;

                    // Get the name from the constant pool
                    let name = match &self.b.var[name_idx].n {
                        E::ST(s) => s.clone(),
                        _ => {
                            return Err(VMError::new(
                                "STORE: expected string name in constant pool",
                            ))
                        }
                    };

                    // Pop the value from the stack
                    let val = self.pop();
                    debug_println!("STORE: {} = {}", name, val);

                    // Store the value in the constant pool and update lookup
                    let val_idx = self.b.var.len() as u16;
                    self.b.var.push(val);
                    self.b.lookup.insert(name, val_idx);
                }
                OP::LOAD(_) => {
                    debug_println!("\n\n-------- LOAD --------");
                    let name_idx = u8_to_u(self.b.op[ip], self.b.op[ip + 1]);
                    ip += 2;

                    // Get the name from the constant pool
                    let name = match &self.b.var[name_idx].n {
                        E::ST(s) => s.clone(),
                        _ => {
                            return Err(VMError::new("LOAD: expected string name in constant pool"))
                        }
                    };

                    // Look up the value
                    let val_idx = self
                        .b
                        .lookup
                        .get(&name)
                        .ok_or_else(|| VMError::new(format!("undefined variable: {}", name)))?
                        .to_owned();
                    self.push(self.b.var[val_idx as usize].clone());
                }
                OP::CALL(_) => {
                    debug_println!("\n\n-------- CALL --------");
                    let _nargs = u8_to_u(self.b.op[ip], self.b.op[ip + 1]);
                    // ip += 2; // would advance past args, but we return early
                    // TODO: implement function call
                    return Err(VMError::new("CALL not yet implemented"));
                }
                OP::MCALL(_) => {
                    debug_println!("\n\n-------- MCALL --------");
                    let name_idx = u8_to_u(self.b.op[ip], self.b.op[ip + 1]);
                    ip += 2;

                    let (nparams, body_op, body_var) = self.resolve_ufnv(name_idx, "MCALL")?;
                    if nparams != 1 {
                        return Err(VMError::new(format!(
                            "MCALL: function expects {} args, got 1",
                            nparams
                        )));
                    }

                    // In a DBL context, temporarily hide the extra duplicated value
                    let dbl_stashed = match self.cget().t {
                        BL::DBL => Some(self.s.pop().expect("stack underflow")),
                        _ => None,
                    };

                    // Pop the arg, run the function body, push the result
                    let arg = self.pop();
                    let result = Self::run_ufnv(&body_op, &body_var, vec![arg])?;
                    self.push(result);

                    // Restore stashed value
                    if let Some(stashed) = dbl_stashed {
                        self.s.push(stashed);
                    }
                }
                OP::DCALL(_) => {
                    debug_println!("\n\n-------- DCALL --------");
                    let name_idx = u8_to_u(self.b.op[ip], self.b.op[ip + 1]);
                    ip += 2;

                    let (nparams, body_op, body_var) = self.resolve_ufnv(name_idx, "DCALL")?;
                    if nparams != 2 {
                        return Err(VMError::new(format!(
                            "DCALL: function expects {} args, got 2",
                            nparams
                        )));
                    }

                    // Pop both args (lhs on top, rhs below)
                    let lhs = self.pop();
                    let rhs = self.pop();
                    let result = Self::run_ufnv(&body_op, &body_var, vec![lhs, rhs])?;
                    self.push(result);

                    // Dyadic ops dup the result for the next train element
                    self.dup();
                }
                _ => return Err(VMError::new(format!("unimplemented instruction: {:?}", op))),
            }
        }
        Ok(())
    }

    /// Resolve a constant-pool name index to a UFNV, returning (nparams, body_op, body_var).
    fn resolve_ufnv(
        &self,
        name_idx: usize,
        ctx: &str,
    ) -> Result<(usize, Vec<u8>, Vec<NN>), VMError> {
        let name = match &self.b.var[name_idx].n {
            E::ST(s) => s.clone(),
            _ => {
                return Err(VMError::new(format!(
                    "{}: expected string name in constant pool",
                    ctx
                )))
            }
        };
        let val_idx = self
            .b
            .lookup
            .get(&name)
            .ok_or_else(|| VMError::new(format!("undefined variable: {}", name)))?
            .to_owned();
        let fn_val = &self.b.var[val_idx as usize];
        match &fn_val.n {
            E::UFNV {
                nparams,
                body_op,
                body_var,
            } => Ok((*nparams, body_op.clone(), body_var.clone())),
            _ => Err(VMError::new(format!(
                "{}: {} is not a function, got {}",
                ctx,
                name,
                type_name(fn_val)
            ))),
        }
    }

    /// Run a UFNV's body bytecode with the given arguments.
    /// Arguments are placed on the stack (in order) before execution.
    /// The body's STORE instructions will pop them and bind to param names.
    /// Returns the last value left on the stack.
    fn run_ufnv(body_op: &[u8], body_var: &[NN], args: Vec<NN>) -> VmRes {
        use std::collections::HashMap;

        let b = B {
            op: body_op.to_vec(),
            var: body_var.to_vec(),
            lookup: HashMap::new(),
            code: HashMap::new(),
        };
        let mut vm = V::new(b);
        // Push args onto the stack (the body's STORE instructions will pop them)
        for arg in args {
            vm.push(arg);
        }
        vm.run()
            .map_err(|e| VMError::new(format!("in user function: {}", e.msg)))?;
        // The result is the last value on the stack
        Ok(vm.pop())
    }

    pub fn cmo(&mut self, co: Option<CN>, fun: FN, _ip: usize) -> Result<(), VMError> {
        match co {
            None => {
                let rhs = self.pop();
                debug_println!("cmo: rhs: {}", rhs);
                let (fun, _) = Self::get_fun(fun);
                let result = fun(&rhs)?;
                self.push(result);
            }
            Some(CN::Fold) => {
                let rhs = self.pop();
                debug_println!("cmo: rhs: {}", rhs);
                let (_, fun) = Self::get_fun(fun);
                match rhs.n {
                    E::LIST(l) => {
                        let mut iter = l.into_iter();
                        let first = iter
                            .next()
                            .ok_or_else(|| VMError::new("fold on empty list"))?;
                        let result = iter.try_fold(first, |acc, a| fun(&acc, &a))?;
                        self.push(result);
                    }
                    _ => {
                        return Err(VMError::new(format!(
                            "fold (/) expects a list, got {}",
                            type_name(&rhs)
                        )));
                    }
                };
            }
            Some(other) => {
                return Err(VMError::new(format!(
                    "combinator {} not supported in monadic context",
                    other
                )));
            }
        }
        Ok(())
    }

    pub fn cdo(&mut self, co: Option<CN>, fun: FN, _ip: usize) -> Result<(), VMError> {
        match co {
            None => {
                debug_println!("cdo None");
                let lhs = self.pop();
                let rhs = self.pop();
                debug_println!("cdo lhs: {}", lhs);
                debug_println!("cdo rhs: {}", rhs);

                let (_, fun) = Self::get_fun(fun);
                let result = fun(&lhs, &rhs)?;
                self.push(result);
            }
            Some(CN::ScanL) => {
                debug_println!("cdo ScanL");
                let lhs = self.pop();
                let rhs = self.pop();
                debug_println!("cdo rhs: {}", rhs);
                debug_println!("cdo lhs: {}", lhs);

                let (_, fun) = Self::get_fun(fun);

                match lhs.n {
                    E::LIST(l) => {
                        let results: Result<Vec<NN>, VMError> = l
                            .into_iter()
                            .map(|w| match rhs.n.clone() {
                                E::LIST(r) => {
                                    let inner: Result<Vec<NN>, VMError> =
                                        r.into_iter().map(|a| fun(&w.clone(), &a)).collect();
                                    Ok(NN::nd(E::LIST(inner?)))
                                }
                                E::INT(_) => fun(&w, &rhs.clone()),
                                _ => Err(VMError::new(format!(
                                    "scan-left (\\) rhs: expected list or int, got {}",
                                    type_name(&rhs)
                                ))),
                            })
                            .collect();
                        self.push(NN::nd(E::LIST(results?)));
                    }
                    _ => {
                        return Err(VMError::new(format!(
                            "scan-left (\\) lhs: expected list, got {}",
                            type_name(&lhs)
                        )));
                    }
                }
            }
            Some(other) => {
                return Err(VMError::new(format!(
                    "combinator {} not supported in dyadic context",
                    other
                )));
            }
        }
        Ok(())
    }

    pub fn get_fun(fun: FN) -> (MonadicFn, DyadicFn) {
        match fun {
            FN::Bang => (mo_bang, do_mathmod),
            FN::Eq => (mo_eq, do_noimpl),
            FN::Div => (mo_noimpl, do_noimpl),
            FN::Max => (mo_noimpl, do_max),
            FN::Min => (mo_min, do_min),
            FN::Amp => (mo_noimpl, do_amp),
            FN::Plus => (mo_noimpl, do_plus),
            FN::Minus => (mo_minus, do_minus),
            FN::Mult => (mo_noimpl, do_noimpl),
            FN::Rho => (mo_rho, do_rho),
        }
    }

    pub fn get_usize(&mut self, ip: usize) -> usize {
        return u8_to_u(self.b.op[ip + 1], self.b.op[ip + 2]);
    }

    pub fn push(&mut self, node: NN) {
        self.s.push(node);
    }

    pub fn pop(&mut self) -> NN {
        let node = self.s.pop().expect("stack underflow");
        self.last_popped = Some(node.clone());
        node
    }

    pub fn get(&mut self) -> NN {
        self.s.last().expect("stack underflow").clone()
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
        let node = self.context[self.cptr - 1].clone();
        self.cptr -= 1;
        node
    }

    pub fn dup(&mut self) {
        debug_println!("!!!!!! DUP !!!!!!");
        let node = self.s.last().expect("stack underflow").clone();
        debug_println!("NODE IS: {}", node);
        self.push(node);
    }

    pub fn ddup(&mut self) {
        debug_println!("!!!!!! DDUP !!!!!!");
        let len = self.s.len();
        let node = self.s[len - 2].clone();
        debug_println!("NODE IS: {}", node);
        self.push(node);
    }

    pub fn pop_last(&self) -> Option<&NN> {
        self.last_popped.as_ref()
    }
}

// ---------------------------------------------------------------------------
// Placeholder for unimplemented monadic / dyadic ops
// ---------------------------------------------------------------------------

pub fn mo_noimpl(rhs: &NN) -> VmRes {
    Err(VMError::new(format!(
        "monadic operation not implemented for {}",
        type_name(rhs)
    )))
}

pub fn do_noimpl(lhs: &NN, rhs: &NN) -> VmRes {
    Err(VMError::new(format!(
        "dyadic operation not implemented for {} and {}",
        type_name(lhs),
        type_name(rhs)
    )))
}

// ---------------------------------------------------------------------------
// Monadic built-ins
// ---------------------------------------------------------------------------

pub fn mo_bang(rhs: &NN) -> VmRes {
    match rhs.n {
        E::INT(i) => Ok(NN::nd(E::LIST(
            (0..i).map(|el| NN::nd(E::INT(el))).collect(),
        ))),
        _ => Err(VMError::new(format!(
            "! (range) expects int, got {}",
            type_name(rhs)
        ))),
    }
}

pub fn mo_minus(rhs: &NN) -> VmRes {
    match rhs.n {
        E::INT(i) => Ok(NN::nd(E::INT(-i))),
        E::FT(f) => Ok(NN::nd(E::FT(-f))),
        _ => Err(VMError::new(format!(
            "- (negate) expects int or float, got {}",
            type_name(rhs)
        ))),
    }
}

pub fn mo_eq(rhs: &NN) -> VmRes {
    match rhs.clone().n {
        E::INT(i) => Ok(NN::nd(E::BOOL(i == 0))),
        E::LIST(l) => {
            let results: Result<Vec<NN>, VMError> = l
                .into_iter()
                .map(|a| match a.n {
                    E::INT(i) => Ok(NN::nd(E::BOOL(i == 0))),
                    E::LIST(_) => mo_eq(&a),
                    _ => Err(VMError::new(format!(
                        "= (boolean flip) expects int inside list, got {}",
                        type_name(&a)
                    ))),
                })
                .collect();
            Ok(NN::nd(E::LIST(results?)))
        }
        _ => Err(VMError::new(format!(
            "= (boolean flip) expects int or list, got {}",
            type_name(rhs)
        ))),
    }
}

pub fn mo_min(rhs: &NN) -> VmRes {
    match rhs.clone().n {
        E::FT(i) => Ok(NN::nd(E::INT(i.floor() as i32))),
        _ => Err(VMError::new(format!(
            "_ (floor) expects float, got {}",
            type_name(rhs)
        ))),
    }
}

// ---------------------------------------------------------------------------
// Dyadic helpers
// ---------------------------------------------------------------------------

pub fn bool_to_int(b: bool) -> i32 {
    match b {
        true => 1,
        false => 0,
    }
}

pub fn do_conversion(lhs: &NN, rhs: &NN, do_target: DyadicFn) -> VmRes {
    match (lhs.clone().n, rhs.clone().n) {
        (E::INT(_), E::INT(_)) => do_target(lhs, rhs),
        (E::BOOL(_), E::BOOL(_)) => do_target(lhs, rhs),
        (E::LIST(l), E::LIST(r)) => {
            if l.len() != r.len() {
                return Err(VMError::new(format!(
                    "list length mismatch: {} vs {}",
                    l.len(),
                    r.len()
                )));
            }
            let results: Result<Vec<NN>, VMError> =
                l.par_iter().zip(r).map(|(l, r)| do_target(l, &r)).collect();
            Ok(NN::nd(E::LIST(results?)))
        }
        _ => Err(VMError::new(format!(
            "type mismatch: {} and {}",
            type_name(lhs),
            type_name(rhs)
        ))),
    }
}

// ---------------------------------------------------------------------------
// Dyadic built-ins
// ---------------------------------------------------------------------------

pub fn do_mathmod(lhs: &NN, rhs: &NN) -> VmRes {
    match (&lhs.n, &rhs.n) {
        (E::INT(w), E::INT(a)) => {
            if *w == 0 {
                return Err(VMError::new("modulo by zero"));
            }
            Ok(NN::nd(E::INT(a % w)))
        }
        _ => do_conversion(lhs, rhs, do_mathmod),
    }
}

pub fn do_mathdiv(lhs: &NN, rhs: &NN) -> VmRes {
    match (&lhs.n, &rhs.n) {
        (E::INT(w), E::INT(a)) => {
            if *w == 0 {
                return Err(VMError::new("division by zero"));
            }
            Ok(NN::nd(E::FT((*a as f64) / (*w as f64))))
        }
        _ => do_conversion(lhs, rhs, do_mathdiv),
    }
}

pub fn do_max(lhs: &NN, rhs: &NN) -> VmRes {
    match (&lhs.n, &rhs.n) {
        (E::INT(w), E::INT(a)) => Ok(NN::nd(E::INT(cmp::max(*a, *w)))),
        (E::BOOL(w), E::BOOL(a)) => Ok(NN::nd(E::BOOL(*w || *a))),
        _ => do_conversion(lhs, rhs, do_max),
    }
}

pub fn do_min(lhs: &NN, rhs: &NN) -> VmRes {
    match (&lhs.n, &rhs.n) {
        (E::INT(w), E::INT(a)) => Ok(NN::nd(E::INT(cmp::min(*a, *w)))),
        _ => do_conversion(lhs, rhs, do_min),
    }
}

pub fn do_amp(lhs: &NN, rhs: &NN) -> VmRes {
    match (lhs.clone().n, rhs.clone().n) {
        (E::LIST(l), E::LIST(a)) => {
            if l.len() != a.len() {
                return Err(VMError::new(format!(
                    "& (filter) list length mismatch: {} vs {}",
                    l.len(),
                    a.len()
                )));
            }
            Ok(NN::nd(E::LIST(l.into_iter().enumerate().fold(
                Vec::new(),
                |mut r: Vec<NN>, (i, w)| match w.n {
                    E::BOOL(true) => {
                        r.push(a[i].clone());
                        r
                    }
                    _ => r,
                },
            ))))
        }
        _ => Err(VMError::new(format!(
            "& (filter) expects two lists, got {} and {}",
            type_name(lhs),
            type_name(rhs)
        ))),
    }
}

pub fn do_plus(lhs: &NN, rhs: &NN) -> VmRes {
    match (&lhs.n, &rhs.n) {
        (E::INT(w), E::INT(a)) => Ok(NN::nd(E::INT(w + a))),
        (E::BOOL(w), E::BOOL(a)) => Ok(NN::nd(E::INT(bool_to_int(*w) + bool_to_int(*a)))),
        _ => do_conversion(lhs, rhs, do_plus),
    }
}

pub fn do_minus(lhs: &NN, rhs: &NN) -> VmRes {
    match (&lhs.n, &rhs.n) {
        (E::INT(w), E::INT(a)) => Ok(NN::nd(E::INT(w - a))),
        _ => do_conversion(lhs, rhs, do_minus),
    }
}

// ---------------------------------------------------------------------------
// Shape / Reshape (ρ)
// ---------------------------------------------------------------------------

/// Extract a shape (Vec<usize>) from an NN that is either a single int or a list of ints.
fn nn_to_shape(nn: &NN) -> Result<Vec<usize>, VMError> {
    match &nn.n {
        E::INT(i) => {
            if *i < 0 {
                return Err(VMError::new(format!(
                    "ρ: shape dimension must be non-negative, got {}",
                    i
                )));
            }
            Ok(vec![*i as usize])
        }
        E::LIST(l) => {
            let mut shape = Vec::with_capacity(l.len());
            for el in l {
                match &el.n {
                    E::INT(i) => {
                        if *i < 0 {
                            return Err(VMError::new(format!(
                                "ρ: shape dimension must be non-negative, got {}",
                                i
                            )));
                        }
                        shape.push(*i as usize);
                    }
                    _ => {
                        return Err(VMError::new(format!(
                            "ρ: shape must be ints, got {}",
                            type_name(el)
                        )))
                    }
                }
            }
            Ok(shape)
        }
        _ => Err(VMError::new(format!(
            "ρ: expected int or list of ints for shape, got {}",
            type_name(nn)
        ))),
    }
}

/// Build a nested LIST structure from flat data and a shape.
fn build_nested(shape: &[usize], data: &[NN]) -> NN {
    if shape.len() <= 1 {
        return NN::nd(E::LIST(data.to_vec()));
    }

    let rows = shape[0];
    let inner_shape = &shape[1..];
    let row_len: usize = inner_shape.iter().product();

    let mut result = Vec::with_capacity(rows);
    for r in 0..rows {
        let start = r * row_len;
        let end = (start + row_len).min(data.len());
        let row_data = if start < data.len() {
            &data[start..end]
        } else {
            &[]
        };
        result.push(build_nested(inner_shape, row_data));
    }
    NN::nd(E::LIST(result))
}

/// Monadic ρ: take a shape description, produce a zeroed array of that shape.
pub fn mo_rho(rhs: &NN) -> VmRes {
    let mut shape = nn_to_shape(rhs)?;
    shape.reverse();
    let total: usize = shape.iter().product();
    let data: Vec<NN> = vec![NN::nd(E::INT(0)); total];
    Ok(build_nested(&shape, &data))
}

/// Dyadic ρ: reshape.
pub fn do_rho(lhs: &NN, rhs: &NN) -> VmRes {
    let mut shape = nn_to_shape(lhs)?;
    shape.reverse();
    let total: usize = shape.iter().product();

    let source: Vec<NN> = match &rhs.n {
        E::LIST(l) => l.clone(),
        _ => vec![rhs.clone()],
    };

    if source.is_empty() {
        return Err(VMError::new("ρ: cannot reshape empty data"));
    }

    let data: Vec<NN> = (0..total)
        .map(|i| source[i % source.len()].clone())
        .collect();

    Ok(build_nested(&shape, &data))
}
