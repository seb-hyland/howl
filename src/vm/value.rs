use crate::parser::Literal;
use std::f64;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Value(u64);

const NAN_MASK: u64 = 0x7FF0_0000_0000_0000;
const FLOAT_NAN: u64 = 0x7FF8_0000_0000_0000;
const PTR: u64 = 0xFFFC_0000_0000_0000;
const INT: u64 = 0xFFFD_0000_0000_0000;
const FALSE: u64 = 0xFFFE_0000_0000_0000;
const TRUE: u64 = 0xFFFE_0000_0000_0001;
const NIL: u64 = 0xFFFE_0000_0000_0002;

impl Value {
    // FROM_ METHODS
    pub fn from_literal(l: Literal) -> Self {
        match l {
            Literal::Int(i, _) => Self::from_int(i),
            Literal::Float(f, _) => Self::from_float(f),
            Literal::Bool(b, _) => Self::from_bool(b),
            Literal::String(s, _) => todo!(),
        }
    }
    pub fn from_float(f: f64) -> Self {
        if f.is_nan() {
            Self(FLOAT_NAN)
        } else {
            Self(f.to_bits())
        }
    }

    pub fn from_int(i: i32) -> Self {
        Self(INT | (i as u64))
    }

    pub fn from_bool(b: bool) -> Self {
        Self(FALSE | (b as u64))
    }

    pub fn nil() -> Self {
        Self(NIL)
    }

    pub fn from_ptr(p: usize) -> Self {
        Self(PTR | (p as u64))
    }

    // IS_ METHODS
    pub fn is_float(&self) -> bool {
        self.0 <= NAN_MASK
    }

    pub fn is_nan(&self) -> bool {
        self.0 == FLOAT_NAN
    }

    pub fn is_int(&self) -> bool {
        (self.0 & 0xFFFF_0000_0000_0000) == INT
    }

    pub fn is_ptr(&self) -> bool {
        (self.0 & 0xFFFF_0000_0000_0000) == PTR
    }

    pub fn is_true(&self) -> bool {
        self.0 == TRUE
    }

    pub fn is_false(&self) -> bool {
        self.0 == FALSE
    }

    pub fn is_nil(&self) -> bool {
        self.0 == NIL
    }

    // AS_ METHODS
    pub fn as_float(&self) -> f64 {
        if self.is_nan() {
            f64::NAN
        } else {
            f64::from_bits(self.0)
        }
    }

    pub fn as_int(&self) -> i32 {
        (self.0 & 0x0000_FFFF_FFFF_FFFF) as i32
    }

    pub fn as_ptr(&self) -> usize {
        (self.0 & 0x0000_FFFF_FFFF_FFFF) as usize
    }

    pub fn type_of(&self) -> usize {
        match self {
            _ if self.is_nil() => 0,
            _ if self.is_ptr() => todo!(),
            _ if self.is_int() => 1,
            _ if self.is_float() => 2,
            _ if self.is_false() => 3,
            _ if self.is_true() => 4,
            _ => unreachable!(),
        }
    }
}

pub enum ValueVariant {
    Float(f64),
    Int(i32),
    Ptr(usize),
    True,
    False,
    Nil,
}
