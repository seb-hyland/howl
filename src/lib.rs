#![feature(maybe_uninit_array_assume_init)]

use crate::vm::globals::Runtime;
use ::std::{cell::UnsafeCell, collections::HashMap, ops::Range, rc::Rc};

pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod std;
pub mod vm;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct Span(Range<usize>);

pub struct StateIterator<'src, T> {
    buf: &'src [T],
    current: UnsafeCell<usize>,
}

impl<'src, T> StateIterator<'src, T> {
    fn new(buf: &'src [T]) -> Self {
        Self {
            buf,
            current: UnsafeCell::new(0),
        }
    }

    #[inline(always)]
    fn current(&self) -> usize {
        unsafe { *self.current.get() }
    }

    #[inline(always)]
    fn incr_current(&self) {
        unsafe { *self.current.get() += 1 }
    }

    fn peek(&self) -> Option<&T> {
        self.buf.get(self.current())
    }

    fn advance(&self) -> Option<&T> {
        let cur_item = self.buf.get(self.current());
        if cur_item.is_some() {
            self.incr_current();
        }
        cur_item
    }

    fn peek_from_current(&self) -> impl IntoIterator<Item = (usize, &T)> {
        self.buf[self.current()..].iter().enumerate()
    }

    fn slice_advance_to_end(&self) -> &[T] {
        let current = self.current();
        let end_idx = self.buf.len();
        unsafe { *self.current.get() = end_idx };
        &self.buf[current..end_idx]
    }

    fn slice_advance(&self, count: usize) -> &[T] {
        if count == 0 {
            todo!()
        }
        let current = self.current();

        let slice = &self.buf[current..current + count];
        unsafe { *self.current.get() += count };
        slice
    }
}

#[derive(Default, Debug)]
pub struct IdentArena {
    map: HashMap<Rc<str>, usize>,
    vec: Vec<Rc<str>>,
}

impl IdentArena {
    pub fn add(&mut self, v: &[u8]) -> usize {
        let s = unsafe { str::from_utf8_unchecked(v) };
        let rc_s = Rc::from(s);

        if let Some(&id) = self.map.get(&rc_s) {
            return id;
        }

        self.vec.push(Rc::clone(&rc_s));
        let id = self.vec.len() - 1;
        self.map.insert(rc_s, id);
        id
    }

    pub fn get(&self, id: usize) -> Option<Rc<str>> {
        self.vec.get(id).map(Rc::clone)
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

pub fn eval(s: &str, rt: &mut Runtime) {
    let tokens = match lexer::lex(s.as_bytes(), &mut rt.globals.idents) {
        Ok(v) => v,
        Err(e) => {
            e.emit(s);
            panic!();
        }
    };
    let stmts = match parser::parse(&tokens) {
        Ok(v) => v,
        Err(_) => panic!(),
    };
    for stmt in stmts {
        compiler::compile_stmt(stmt, rt);
    }
    vm::runtime::exe(rt);
}
