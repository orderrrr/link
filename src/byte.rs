use std::collections::HashMap;

use debug_print::debug_println;

use crate::err::LErrEnum::ExprExpected as Er;
use crate::{LErrEnum, LocatedError, URes, VURes, get_fnop, get_cnop};

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

#[derive(Debug, Clone)]
pub struct B {
    pub op: Vec<u8>,
    // operations
    pub var: Vec<NN>,
    // variables
    pub lookup: HashMap<String, u16>,
    // lookup
    pub code: HashMap<u16, (usize, usize)>, // code
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
            match n.n {
                E::ASEXP { op: _, rhs: _ } => return,
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
        // kind of unsafe
        let u8: [u8; 2] = u16_to_u8(j);
        self.b.op[i as usize + 1] = u8[0];
        self.b.op[i as usize + 2] = u8[1];
    }

    fn addmap(&mut self, n: NN, ci: u16) {
        match n.n {
            E::VAL(s) => self.b.lookup.insert(s, ci),
            _ => unreachable!("unreachable"),
        };
    }

    fn irvar(&mut self, n: NN) -> URes {
        match n.n {
            E::INT(_) | E::FT(_) | E::ST(_) | E::LIST(_) => {
                let ci = self.addvar(n.clone());
                self.addlookup("a", ci);
                Ok(ci.to_owned())
            }
            E::VAL(x) => {
                let ci = self.b.lookup.get(&x).ok_or(Er)?;
                Ok(ci.to_owned())
            }
            _ => unreachable!("unreachable got: {}", n),
        }
    }

    fn ilvar(&mut self, n: NN) -> URes {
        match n.n {
            E::INT(_) | E::FT(_) | E::ST(_) | E::LIST(_) => {
                let ci = self.addvar(n.clone());
                self.addlookup("w", ci);
                Ok(ci)
            }
            E::VAL(x) => {
                let ci = self.b.lookup.get(&x).ok_or(Er)?;
                Ok(ci.to_owned())
            }
            _ => unreachable!("unreachable"),
        }
    }

    fn inode(&mut self, n: NN, s: Option<SET>) -> URes {
        let set = s.unwrap_or_default();
        println!("CURRENT NODE: {}", n.clone());
        match n.clone().n {
            E::INT(_) | E::FT(_) | E::ST(_) | E::LIST(_) => {
                // let ci = self.addvar(n.clone());
                // self.addop(n, OP::CONST(ci))
                let ci = self.addvar(n.clone());
                self.addop(n, OP::CONST(ci))
            }
            E::VAL(v) => {
                // match v {
                //     "w" | "a" => self.addop(n, op)
                // }
                println!("v is: {}", v);
                println!("lookup is: {:?}", self.b.lookup);

                match v.as_str() {
                    "w" => self.addop(n, OP::GETL),
                    "r" => self.addop(n, OP::GETR),
                    _ => {
                        let ci = self.b.lookup.get(&v).unwrap().to_owned();
                        self.addop(n, OP::CONST(ci))
                    }
                }
            }
            E::FVAL(v) => {
                let ci = self.b.lookup.get(&v).unwrap().to_owned();
                self.addop(n, OP::JMP(ci))
            }
            E::ASEXP { op, rhs } => {
                let blp = self.addop(n.clone(), OP::JMP(0))? - 3;

                match rhs.n {
                    E::FBLOCK(_) | E::TFBLOCK(_) | E::TMFBLOCK(_) => self.inode(*rhs, None),
                    _ => unreachable!("unreachable"),
                }?;
                self.addmap(*op, blp + 3);

                let end = self.addop(n, OP::END)?;
                self.upop(end, blp);
                Ok(end)
            }
            E::MEXP { op, rhs } => {
                let r = self.irvar(*rhs.clone())?;
                self.addop(*rhs, OP::CONST(r))?;

                let blp = self.addop(n.clone(), OP::MBL(0))?;

                op.into_iter()
                        .rev()
                        .map(|o| self.inode(o, None))
                        .last()
                        .ok_or(Er)??;

                let end = self.addop(n, OP::END)?;
                self.upop(end, blp - 3);
                Ok(end)
            }
            E::DEXP { op, lhs, rhs } => {
                let l = self.irvar(*rhs.clone())?;
                let r = self.ilvar(*lhs.clone())?;
                self.addop(*lhs, OP::CONST(l))?;
                self.addop(*rhs, OP::CONST(r))?;

                let blp = self.addop(n.clone(), OP::DBL(0))?;

                op.into_iter()
                        .rev()
                        .map(|o| self.inode(o, Some(SET::DBL)))
                        .last()
                        .ok_or(Er)??;

                let end = self.addop(n, OP::END)?;
                self.upop(end, blp - 3);
                Ok(end)
            }
            E::MFN(_) => {
                self.fnode(n)
            }
            E::DFN(_) => {
                self.fnode(n)
            }
            E::MCO { o, co } => {
                self.fnode(*o)?;
                self.fnode(*co)
            }
            E::DCO { o, co } => {
                self.fnode(*o)?;
                self.fnode(*co)
            }
            E::BL(block) => {
                let blp = self.addop(n.clone(), I::get_op(set, 0))?;

                let end = self.inode(*block, None)?;

                self.upop(end, blp - 2);
                self.addop(n, OP::END)
            }
            E::FBLOCK(block) | E::MFBLOCK(block) => {
                let blp = self.addop(n.clone(), I::get_op(set, 0))?;

                let end = block
                        .into_iter()
                        .rev()
                        .map(|o| self.inode(o, None))
                        .last()
                        .ok_or(Er)??;

                self.upop(end, blp - 3);
                // self.addop(n, OP::END(blp - 2))
                self.addop(n, OP::END)
            }
            E::TFBLOCK(block) => {
                let blp = self.addop(n.clone(), I::get_op(set, 0))?;
                self.addop(n.clone(), OP::GETL)?;
                self.addop(n.clone(), OP::GETR)?;
                let end = block
                        .into_iter()
                        .rev()
                        .map(|o| self.inode(o, None))
                        .last()
                        .ok_or(Er)??;
                self.upop(end, blp - 3);
                self.addop(n, OP::END)
            }
            E::TMFBLOCK(block) => {
                let blp = self.addop(n.clone(), I::get_op(set, 0))?;
                self.addop(n.clone(), OP::GETR)?;
                let end = block
                        .into_iter()
                        .rev()
                        .map(|o| self.inode(o, None))
                        .last()
                        .ok_or(Er)??;
                self.upop(end, blp - 3);
                self.addop(n, OP::END)
            }
            E::MTRAIN(train) | E::DTRAIN(train) => train
                    .into_iter()
                    .rev()
                    .map(|t| self.inode(t, None))
                    .last()
                    .ok_or(Er)?,
            E::DDTRAIN { op, lhs } => {
                let blp = self.addop(n.clone(), OP::DBL(0))?;
                self.inode(*lhs, Some(SET::DBL))?;
                op.into_iter()
                        .rev()
                        .map(|o| self.inode(o, None))
                        .collect::<VURes>()?;
                let end = self.addop(n, OP::END);
                self.upop(end.clone()?, blp - 3);
                end
                // self.addop(n, OP::CLVAR)
            }
            E::DBLOCK { op, lhs } => {
                let blp = self.addop(n.clone(), I::get_op(SET::DBL, 0))?;

                // dup needs to have start and end
                let dlp = self.addop(n.clone(), OP::DUP(0))?;
                self.inode(*lhs, None)?;
                let dend = self.addop(n.clone(), OP::END)?;
                self.upop(dend.clone(), dlp -3);

                op
                        .into_iter()
                        .rev()
                        .map(|o| self.inode(o, None))
                        .collect::<VURes>()?
                        .last()
                        .ok_or(Er)?;

                let end = self.addop(n, OP::END);
                self.upop(end.clone()?, blp - 3);
                end
            }
            E::CN(_) | E::BOOL(_) => {
                unreachable!("should not see this here")
            }
        }
    }

    fn fnode(&mut self, n: NN) -> URes {
        match n.n {
            E::MFN(fun) => self.addop(n, OP::MO(get_fnop(fun))),
            E::DFN(fun) => self.addop(n, OP::DO(get_fnop(fun))),
            E::CN(fun) => self.addop(n, OP::CO(get_cnop(fun))),
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
