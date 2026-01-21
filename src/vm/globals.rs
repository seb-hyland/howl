use crate::{IdentArena, parser::Ident, vm::value::Value};
use std::{collections::HashMap, mem::MaybeUninit};

#[derive(Default)]
pub struct Runtime {
    pub globals: Globals,
    pub stack: Vec<Value>,
    pub code: Vec<OpCode>,
    pub pc: usize,
}

impl Runtime {
    pub fn new() -> Self {
        let mut rt = Self::default();
        crate::std::define_std_types(&mut rt);
        rt
    }
}

#[derive(Default)]
pub struct Globals {
    pub idents: IdentArena,
    pub vars: Vec<Option<Value>>,
    pub types: HashMap<usize, Type>,
}

#[derive(Default)]
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

    pub fn peek_at(&self, n: usize) -> Value {
        let len = self.stack.len();
        if len <= n {
            panic!(
                "Stack underflow during peek_at: requested {}, but stack size is {}",
                n, len
            );
        }
        self.stack[len - 1 - n]
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

    pub fn register_handler(&mut self, name: &'static str, handler: ExternHandler, ty: &mut Type) {
        let handler_id = self.globals.idents.add(name.as_bytes());
        ty.handlers.insert(handler_id, Handler::Extern(handler));
    }

    pub fn push_op(&mut self, op: OpCode) {
        self.code.push(op);
    }
}

type ExternHandler = fn(rt: &mut Runtime, arg_count: usize) -> Option<Value>;
pub enum Handler {
    Extern(ExternHandler),
}

#[derive(Debug)]
pub enum OpCode {
    PushLit(Value),
    PushGlobal(usize),
    SetGlobal(usize),
    SendMessage { id: usize, arg_count: usize },
}
