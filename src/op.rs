use crate::ast::{CN, FN}; // 0.17.1

const CONST: u8 = 1;
const POP: u8 = 2;
const JMP: u8 = 3;
const GETL: u8 = 4;
const GETR: u8 = 5;
const CRVAR: u8 = 6;
const CLVAR: u8 = 7;
const DUP: u8 = 8;
const MBL: u8 = 9;
const DBL: u8 = 10;
const END: u8 = 11;
const MO: u8 = 12;
const DO: u8 = 13;
const CO: u8 = 14;

const FNPLUS: u8 = 1;
const FNMINUS: u8 = 2;
const FNMAX: u8 = 3;
const FNMULT: u8 = 8;
const FNDIV: u8 = 9;
const FNMIN: u8 = 4;
const FNEQ: u8 = 5;
const FNAMP: u8 = 6;
const FNBANG: u8 = 7;

const COFOLD: u8 = 1;
const COSCANL: u8 = 2;
const COEACH: u8 = 3;

#[derive(Debug, Copy, Clone)]
pub enum OP {
    CONST(u16), // pointer to constant table
    POP,
    JMP(u16),

    GETL,
    GETR,

    CRVAR,
    CLVAR,

    DUP(u16),

    MBL(u16),
    DBL(u16),
    END,

    MO(u8),
    DO(u8),
    CO(u8),
}

pub fn u16_to_u8(integer: u16) -> [u8; 2] {
    [(integer >> 8) as u8, integer as u8]
}

pub fn u8_to_u(int1: u8, int2: u8) -> usize {
    ((int1 as usize) << 8) | int2 as usize
}

fn make_o(code: u8, data: u16) -> Vec<u8> {
    let mut output = vec![code];
    output.extend(&u16_to_u8(data));
    output
}

pub fn get_op(op: OP) -> u8 {
    match op {
        OP::CONST(_) => CONST,

        OP::POP => POP, // decimal repr is 2

        OP::JMP(_) => JMP,

        OP::GETL => GETL,
        OP::GETR => GETR,
        OP::CRVAR => CRVAR,
        OP::CLVAR => CLVAR,

        OP::DUP(_) => DUP,

        OP::MBL(_) => MBL,
        OP::DBL(_) => DBL,
        OP::END => END,

        OP::MO(_) => MO,
        OP::DO(_) => DO,
        OP::CO(_) => CO,
    }
}

pub fn get_fnop(fun: FN) -> u8 {
    match fun {
        FN::Plus => FNPLUS,
        FN::Minus => FNMINUS,
        FN::Mult => FNMULT,
        FN::Div => FNDIV,
        FN::Max => FNMAX,
        FN::Min => FNMIN,
        FN::Eq => FNEQ,
        FN::Amp => FNAMP,
        FN::Bang => FNBANG,
    }
}

pub fn get_cnop(cn: CN) -> u8 {
    match cn {
        CN::Fold => COFOLD,
        CN::ScanL => COSCANL,
        CN::Each => COEACH,
    }
}

pub fn make_op(op: OP) -> Vec<u8> {
    let code = get_op(op);
    match op {
        OP::POP | OP::CLVAR | OP::CRVAR | OP::GETL | OP::GETR | OP::END => {
            vec![code]
        } // decimal repr is 2

        OP::MO(a) => vec![MO, a],
        OP::DO(a) => vec![DO, a],
        OP::CO(a) => vec![CO, a],

        OP::CONST(a) | OP::JMP(a) | OP::MBL(a) | OP::DBL(a) | OP::DUP(a) => make_o(code, a),
    }
}

pub fn byte_to_op(by: u8) -> Option<OP> {
    match by {
        CONST => Some(OP::CONST(0)),
        POP => Some(OP::POP),
        JMP => Some(OP::JMP(0)),
        GETL => Some(OP::GETL),
        GETR => Some(OP::GETR),
        CRVAR => Some(OP::CRVAR),
        CLVAR => Some(OP::CLVAR),
        DUP => Some(OP::DUP(0)),
        MBL => Some(OP::MBL(0)),
        DBL => Some(OP::DBL(0)),
        END => Some(OP::END),

        MO => Some(OP::MO(0)),
        DO => Some(OP::DO(0)),
        CO => Some(OP::CO(0)),

        _ => None,
    }
}

pub fn byte_to_fn(by: u8) -> FN {
    match by {
        FNPLUS => FN::Plus,
        FNMINUS => FN::Minus,
        FNMULT => FN::Mult,
        FNDIV => FN::Div,
        FNMAX => FN::Max,
        FNMIN => FN::Min,
        FNEQ => FN::Eq,
        FNAMP => FN::Amp,
        FNBANG => FN::Bang,
        _ => unreachable!("unknown fn code"),
    }
}

pub fn byte_to_co(by: u8) -> CN {
    match by {
        COSCANL => CN::ScanL,
        COFOLD => CN::Fold,
        COEACH => CN::Each,
        _ => unreachable!("Expected CN"),
    }
}

pub fn op_to_co(op: OP, co: u8) -> Option<CN> {
    match op {
        OP::CO(_) => Some(byte_to_co(co)),
        _ => None,
    }
}
