use crate::{parser::Literal, vm::runtime::Heap};
use std::{f64, ptr::NonNull};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct Value(u64);

const NAN_MASK: u64 = 0x7FF0_0000_0000_0000;
const FLOAT_NAN: u64 = 0x7FF8_0000_0000_0000;
const PTR: u64 = 0xFFFC_0000_0000_0000;
const INT: u64 = 0xFFFD_0000_0000_0000;
const FALSE: u64 = 0xFFFE_0000_0000_0000;
const TRUE: u64 = 0xFFFE_0000_0000_0001;
const NIL: u64 = 0xFFFE_0000_0000_0002;
const HIGH_MASK: u64 = 0xFFFF_0000_0000_0000;

impl Value {
    // FROM_ METHODS
    pub fn from_literal(l: Literal, heap: &mut Heap) -> Self {
        match l {
            Literal::Int(i) => Self::from_int(i),
            Literal::Float(f) => Self::from_float(f),
            Literal::Bool(b) => Self::from_bool(b),
            Literal::Nil => Self::nil(),
            Literal::String(s) => {
                let s_len = s.len() as u64;
                let header_len = 16_u64;

                let ptr = heap.alloc(s_len + header_len, TypeId::String).unwrap();
                unsafe {
                    ptr.cast::<u64>().write(s_len);
                    ptr.add(header_len as usize).copy_from_nonoverlapping(
                        NonNull::new(s.as_ptr() as *mut u8).unwrap(),
                        s_len as usize,
                    );
                };

                Self::from_ptr(ptr.as_ptr() as u64)
            }
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

    pub fn from_uint(i: u64) -> Self {
        Self(INT | i)
    }

    pub fn from_bool(b: bool) -> Self {
        Self(FALSE | (b as u64))
    }

    pub fn nil() -> Self {
        Self(NIL)
    }

    pub fn from_ptr(p: u64) -> Self {
        Self(PTR | p)
    }

    // IS_ METHODS
    pub fn is_float(&self) -> bool {
        self.0 <= NAN_MASK
    }

    pub fn is_nan(&self) -> bool {
        self.0 == FLOAT_NAN
    }

    pub fn is_int(&self) -> bool {
        (self.0 & HIGH_MASK) == INT
    }

    pub fn is_ptr(&self) -> bool {
        (self.0 & HIGH_MASK) == PTR
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

    pub fn as_uint(&self) -> u64 {
        self.0 & 0x0000_FFFF_FFFF_FFFF
    }

    pub fn as_ptr(&self) -> u64 {
        self.0 & 0x0000_FFFF_FFFF_FFFF
    }

    pub fn type_of(&self) -> TypeId {
        match self {
            _ if self.is_nil() => TypeId::Nil,
            _ if self.is_ptr() => unsafe {
                (self.as_ptr() as *const u8).sub(16).cast::<TypeId>().read()
            },
            _ if self.is_int() => TypeId::Int,
            _ if self.is_float() => TypeId::Float,
            _ if self.is_false() => TypeId::False,
            _ if self.is_true() => TypeId::True,
            _ => unreachable!(),
        }
    }
}

#[repr(u64)]
#[derive(Copy, Clone)]
pub enum TypeId {
    NONE,
    // Primitives
    Nil,
    Int,
    Float,
    True,
    False,

    String,
    HeapMap,
    CompiledBytecode,
}
