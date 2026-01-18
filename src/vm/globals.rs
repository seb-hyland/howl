use crate::{lexer::IdentArena, parser::Ident, vm::value::Value};
use std::{
    collections::HashMap,
    mem::{self, MaybeUninit},
    ops::Add,
};

pub struct Runtime {
    pub globals: Globals,
    pub stack: Vec<Value>,
    pub code: Vec<OpCode>,
    pub pc: usize,
}

pub struct Globals {
    pub idents: IdentArena,
    pub vars: Vec<Option<Value>>,
    pub types: HashMap<usize, Type>,
}

pub struct Type {
    pub instance_fields: Vec<Ident>,
    pub type_fields: Vec<Ident>,
    pub handlers: HashMap<usize, Handler>,
}

impl Runtime {
    #[inline(always)]
    pub fn push_stack(&mut self, v: Value) {
        self.stack.push(v);
    }

    #[inline(always)]
    pub fn pop_stack(&mut self) -> Value {
        self.stack.pop().expect("Stack should not be empty")
    }

    pub fn peek(&self) -> &Value {
        self.stack.last().expect("Stack should not be empty")
    }

    #[inline(always)]
    pub fn pop_stack_n<const N: usize>(&mut self) -> [Value; N] {
        let mut array = [MaybeUninit::uninit(); N];
        for i in (0..N).rev() {
            array[i].write(self.pop_stack());
        }
        unsafe { MaybeUninit::array_assume_init(array) }
    }

    pub fn define_type(&mut self, id: usize, t: Type) {
        self.globals.types.insert(id, t);
    }

    pub fn push_op(&mut self, op: OpCode) {
        self.code.push(op);
    }
}

pub enum Handler {
    Extern(fn(rt: &mut Runtime, arg_count: usize) -> Option<Value>),
}

pub enum OpCode {
    PushLiteral(Value),
    PushGlobal(usize),
    SetGlobal(usize),
    SendMessage { id: usize, arg_count: usize },
}
