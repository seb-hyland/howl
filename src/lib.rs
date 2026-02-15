#![feature(maybe_uninit_array_assume_init)]
#![feature(allocator_api)]
#![feature(ptr_as_ref_unchecked)]

use crate::{
    parser::{Stmt, howl_parser},
    vm::runtime::Runtime,
};
use ::std::{collections::HashMap, ops::Range, rc::Rc};
use peg::{error::ParseError, str::LineCol};

pub mod compiler;
pub mod parser;
pub mod std;
pub mod vm;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct Span(Range<usize>);

#[derive(Default, Debug)]
pub struct IdentArena {
    map: HashMap<Rc<str>, u64>,
    vec: Vec<Rc<str>>,
}

impl IdentArena {
    pub fn add(&mut self, s: &str) -> u64 {
        let rc_s = Rc::from(s);

        if let Some(&id) = self.map.get(&rc_s) {
            return id;
        }

        self.vec.push(Rc::clone(&rc_s));
        let id = (self.vec.len() - 1) as u64;
        self.map.insert(rc_s, id);
        id
    }

    pub fn get(&self, id: u64) -> Option<Rc<str>> {
        self.vec.get(id as usize).map(Rc::clone)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

pub fn parse(s: &str, rt: &mut Runtime) -> Result<Vec<Stmt>, ParseError<LineCol>> {
    howl_parser::statements(s, &mut rt.globals.idents)
}

pub fn run(stmts: Vec<Stmt>, rt: &mut Runtime) {
    for stmt in stmts {
        compiler::compile_stmt(stmt, &mut rt.code, &mut rt.heap);
    }
    vm::bytecode::flush_runtime(rt);
}

pub fn eval(s: &str, rt: &mut Runtime) {
    let stmts = howl_parser::statements(s, &mut rt.globals.idents).unwrap();
    for stmt in stmts {
        compiler::compile_stmt(stmt, &mut rt.code, &mut rt.heap);
    }
    vm::bytecode::flush_runtime(rt);
}
