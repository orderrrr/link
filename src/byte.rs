use std::collections::HashMap;

use debug_print::debug_println;

use crate::err::LErrEnum::ExprExpected as Er;
use crate::{get_cnop, get_fnop, LErrEnum, LocatedError, URes};

use crate::{
    ast::{E, NN},
    op::{make_op, u16_to_u8, OP},
    parse::parse,
    BRes,
};

#[derive(Default, Copy, Clone)]
pub enum SET {
    DBL,
    #[default]
    MBL,
}

/// Persistent environment carried across REPL iterations.
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub var: Vec<NN>,
    pub lookup: HashMap<String, u16>,
}

#[derive(Debug, Clone)]
pub struct B {
    pub op: Vec<u8>,
    pub var: Vec<NN>,
    pub lookup: HashMap<String, u16>,
    pub code: HashMap<u16, (usize, usize)>,
}

impl B {
    fn new() -> Self {
        Self {
            op: vec![],
            var: vec![],
            lookup: HashMap::new(),
            code: HashMap::new(),
        }
    }
}

impl PartialEq for B {
    fn eq(&self, other: &Self) -> bool {
        self.op == other.op && self.var == other.var && self.lookup == other.lookup
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct I {
    b: B,
}

#[allow(dead_code)]
impl I {
    fn new() -> Self {
        Self { b: B::new() }
    }

    pub fn fstring(s: &str) -> BRes {
        debug_println!("Compiling the source: {}", s);
        let ast: Vec<NN> = parse(s)?;
        debug_println!("{:?}", ast);
        Self::fast(ast)
    }

    /// Compile a string, seeding the compiler with an existing environment
    /// so that variables defined in prior REPL lines are available.
    pub fn fstring_with_env(s: &str, env: Env) -> BRes {
        debug_println!("Compiling the source (with env): {}", s);
        let ast: Vec<NN> = parse(s)?;
        debug_println!("{:?}", ast);
        Self::fast_with_env(ast, env)
    }

    fn fast(a: Vec<NN>) -> BRes {
        let mut i = I::new();
        let mut e: Option<LocatedError<LErrEnum>> = None;
        a.into_iter().for_each(|n| {
            debug_println!("Compiling node {:?}", n);
            match i.inode(n.clone(), None) {
                Ok(_) => (),
                Err(err) => {
                    e = Some(err);
                    return;
                }
            };
            // Don't add POP after assignments or do-blocks that contain assignments
            match n.n {
                E::ASEXP { .. } => return,
                _ => (),
            };
            match i.addop(n, OP::POP) {
                Ok(_) => (),
                Err(err) => {
                    e = Some(err);
                    return;
                }
            };
        });
        if e.is_some() {
            return Err(e.unwrap());
        }
        Ok(i.b)
    }

    fn fast_with_env(a: Vec<NN>, env: Env) -> BRes {
        let mut i = I::new();
        // Seed the compiler with the prior environment
        i.b.var = env.var;
        i.b.lookup = env.lookup;
        let mut e: Option<LocatedError<LErrEnum>> = None;
        a.into_iter().for_each(|n| {
            debug_println!("Compiling node {:?}", n);
            match i.inode(n.clone(), None) {
                Ok(_) => (),
                Err(err) => {
                    e = Some(err);
                    return;
                }
            };
            match n.n {
                E::ASEXP { .. } => return,
                _ => (),
            };
            match i.addop(n, OP::POP) {
                Ok(_) => (),
                Err(err) => {
                    e = Some(err);
                    return;
                }
            };
        });
        if e.is_some() {
            return Err(e.unwrap());
        }
        Ok(i.b)
    }

    fn addvar(&mut self, n: NN) -> u16 {
        let pos = (n.start, n.end);
        self.b.var.push(n);
        let len = (self.b.var.len() - 1) as u16;
        self.b.code.insert(len, pos);
        len
    }

    fn addlookup(&mut self, v: &str, n: u16) {
        self.b.lookup.insert(v.to_owned(), n);
    }

    fn addop(&mut self, n: NN, op: OP) -> URes {
        let pos = (n.start, n.end);
        self.b.op.extend(make_op(op));
        debug_println!(
            "added instructions {:?} from opcode {:?}",
            self.b.op,
            op.clone()
        );
        let len = self.b.op.len() as u16;
        self.b.code.insert(len, pos);
        Ok(len)
    }

    fn upop(&mut self, j: u16, i: u16) {
        let u8: [u8; 2] = u16_to_u8(j);
        self.b.op[i as usize + 1] = u8[0];
        self.b.op[i as usize + 2] = u8[1];
    }

    fn inode(&mut self, n: NN, s: Option<SET>) -> URes {
        let _set = s.unwrap_or_default();
        debug_println!("CURRENT NODE: {}", n.clone());
        match n.clone().n {
            E::INT(_) | E::FT(_) | E::ST(_) | E::LIST(_) | E::BOOL(_) => {
                // For LIST, recursively compile elements first
                let node = match &n.n {
                    E::LIST(elems) => {
                        // Compile each element, build a runtime LIST
                        let compiled_elems: Result<Vec<NN>, _> = elems
                            .iter()
                            .map(|e| {
                                // For literal elements, just keep them
                                Ok(e.clone())
                            })
                            .collect();
                        NN::nd(E::LIST(
                            compiled_elems.map_err(|_: ()| LocatedError::from(Er))?,
                        ))
                    }
                    _ => n.clone(),
                };
                let ci = self.addvar(node);
                self.addop(n, OP::CONST(ci))
            }

            E::VAL(v) => {
                debug_println!("v is: {}", v);
                debug_println!("lookup is: {:?}", self.b.lookup);
                match self.b.lookup.get(&v) {
                    Some(ci) => {
                        let ci = ci.to_owned();
                        self.addop(n, OP::CONST(ci))
                    }
                    None => {
                        // Variable not yet defined — store as a name for late binding
                        let name_node = NN::nd(E::ST(v.clone()));
                        let ci = self.addvar(name_node);
                        self.addop(n, OP::LOAD(ci))
                    }
                }
            }

            E::ASEXP { name, rhs } => {
                // Compile the rhs
                self.inode(*rhs.clone(), None)?;

                // Store the name in the constant pool
                let name_node = NN::nd(E::ST(name.clone()));
                let name_idx = self.addvar(name_node);

                // After the rhs is on the stack, STORE pops it and binds to name
                self.addop(n, OP::STORE(name_idx))
            }

            E::DOBLOCK(exprs) => {
                let len = exprs.len();
                let mut last = 0u16;
                for (i, expr) in exprs.into_iter().enumerate() {
                    let is_last = i == len - 1;
                    let is_assign = matches!(&expr.n, E::ASEXP { .. });
                    last = self.inode(expr.clone(), None)?;
                    // POP intermediate results (but not assignments, not the last expr)
                    if !is_last && !is_assign {
                        self.addop(expr, OP::POP)?;
                    }
                }
                Ok(last)
            }

            E::LAMBDA { params, body } => {
                // Compile lambda as: JMP past body, then body code, then END
                // The UFNV captures the body bytecode + constants so it is
                // self-contained and works across REPL compilation units.

                // Emit JMP to skip past the body (will be patched)
                let jmp_pos = self.addop(n.clone(), OP::JMP(0))? - 3;

                let nparams = params.len();
                let body_start = (jmp_pos + 3) as usize;

                // Body: bind params from stack, execute body, leave result on stack
                // Params are pushed in order by the caller, so we pop them in reverse
                // NOTE: we intentionally do NOT addlookup for params here. This forces
                // the body's references to params to use LOAD (runtime resolution)
                // instead of CONST (which would push the name string, not the value).
                for param in params.iter().rev() {
                    let name_node = NN::nd(E::ST(param.clone()));
                    let name_idx = self.addvar(name_node);
                    self.addop(n.clone(), OP::STORE(name_idx))?;
                }

                // Compile body expressions
                let body_len = body.len();
                for (i, expr) in body.into_iter().enumerate() {
                    let is_last = i == body_len - 1;
                    let is_assign = matches!(&expr.n, E::ASEXP { .. });
                    self.inode(expr.clone(), None)?;
                    if !is_last && !is_assign {
                        self.addop(expr, OP::POP)?;
                    }
                }

                let end_pos = self.addop(n.clone(), OP::END)?;

                // Patch the JMP to skip to after END
                self.upop(end_pos, jmp_pos);

                // Extract the body bytecode (from body_start up to end_pos)
                let body_op = self.b.op[body_start..end_pos as usize].to_vec();
                // Snapshot the full constant pool so indices remain valid
                let body_var = self.b.var.clone();

                let lambda_node = NN::nd(E::UFNV {
                    nparams,
                    body_op,
                    body_var,
                });
                let lambda_idx = self.addvar(lambda_node);
                self.addop(n, OP::CONST(lambda_idx))
            }

            E::APPLY { train, args } => {
                let nargs = args.len();

                match nargs {
                    1 => {
                        // Monadic: push arg, then apply train
                        let rhs = args.into_iter().next().ok_or(Er)?;
                        self.inode(rhs.clone(), None)?;
                        self.addlookup("a", (self.b.var.len().max(1) - 1) as u16);

                        let blp = self.addop(n.clone(), OP::MBL(0))?;

                        // Compile train in reverse (rightmost applied first)
                        let train_vec: Vec<NN> = train;
                        for t in train_vec.into_iter().rev() {
                            self.compile_train_elem(t, true)?;
                        }

                        let end = self.addop(n, OP::END)?;
                        self.upop(end, blp - 3);
                        Ok(end)
                    }
                    2 => {
                        // Dyadic: push both args, then apply train
                        let mut args_iter = args.into_iter();
                        let lhs = args_iter.next().ok_or(Er)?;
                        let rhs = args_iter.next().ok_or(Er)?;

                        // For dyadic: rhs is "a" (right arg), lhs is "w" (left arg)
                        self.inode(rhs.clone(), None)?;
                        self.addlookup("a", (self.b.var.len().max(1) - 1) as u16);

                        self.inode(lhs.clone(), None)?;
                        self.addlookup("w", (self.b.var.len().max(1) - 1) as u16);

                        let blp = self.addop(n.clone(), OP::DBL(0))?;

                        // Compile train in reverse (right-to-left evaluation).
                        // In a dyadic context with a multi-element train:
                        //   - Rightmost elements apply monadically to rhs
                        //   - Leftmost element applies dyadically (lhs + chain result)
                        // Single-element train: the one op is dyadic.
                        let train_vec: Vec<NN> = train;
                        let train_len = train_vec.len();
                        for (i, t) in train_vec.into_iter().rev().enumerate() {
                            let is_last_in_reverse = i == train_len - 1;
                            if train_len == 1 {
                                // Single-op train: dyadic
                                self.compile_train_elem(t, false)?;
                            } else if is_last_in_reverse {
                                // Leftmost element (last in reverse): dyadic
                                self.compile_train_elem(t, false)?;
                            } else {
                                // Rightmost and intermediate ops: monadic
                                self.compile_train_elem(t, true)?;
                            }
                        }

                        let end = self.addop(n, OP::END)?;
                        self.upop(end, blp - 3);
                        Ok(end)
                    }
                    _ => {
                        // Variadic — not yet supported for built-in trains
                        // Could be a user function call with multiple args
                        // For now, treat as monadic with a list arg
                        Err(LocatedError::from(Er))
                    }
                }
            }

            E::MFN(_)
            | E::DFN(_)
            | E::CN(_)
            | E::MCO { .. }
            | E::DCO { .. }
            | E::MOP(_)
            | E::UFNV { .. } => {
                // These should only appear inside trains, not as top-level nodes
                Err(LocatedError::from(Er))
            }
        }
    }

    /// Compile a single train element (op, combinator, cfn, name, or monadic override)
    fn compile_train_elem(&mut self, t: NN, monadic: bool) -> URes {
        match t.n.clone() {
            E::MFN(fun) => {
                if monadic {
                    self.addop(t, OP::MO(get_fnop(fun)))
                } else {
                    self.addop(t, OP::DO(get_fnop(fun)))
                }
            }
            E::DFN(fun) => self.addop(t, OP::DO(get_fnop(fun))),
            E::CN(cn) => self.addop(t, OP::CO(get_cnop(cn))),
            E::MOP(fun) => {
                // Monadic override: always emit as MO regardless of context
                self.addop(t, OP::MO(get_fnop(fun)))
            }
            E::MCO { o, co } | E::DCO { o, co } => {
                // Operator + combinator pair
                // In monadic context, emit the op as MO then the combinator
                // In dyadic context, emit the op as DO then the combinator
                match monadic {
                    true => {
                        self.compile_train_elem(*o, true)?;
                        self.compile_train_elem(*co, true)
                    }
                    false => {
                        // For dyadic cfn like +/ in a dyadic context,
                        // the operator becomes dyadic
                        match o.n.clone() {
                            E::MFN(fun) => {
                                self.addop(*o, OP::DO(get_fnop(fun)))?;
                            }
                            _ => {
                                self.compile_train_elem(*o, false)?;
                            }
                        }
                        self.compile_train_elem(*co, false)
                    }
                }
            }
            E::VAL(name) => {
                // A name in a train — this is a user function reference
                // Emit MCALL (monadic) or DCALL (dyadic) with the name index
                // so the VM can resolve the function at runtime
                let name_node = NN::nd(E::ST(name));
                let ci = self.addvar(name_node);
                if monadic {
                    self.addop(t, OP::MCALL(ci))
                } else {
                    self.addop(t, OP::DCALL(ci))
                }
            }
            _ => Err(LocatedError::from(Er)),
        }
    }

    fn get_op(s: SET, n: u16) -> OP {
        match s {
            SET::MBL => OP::MBL(n),
            SET::DBL => OP::DBL(n),
        }
    }
}
