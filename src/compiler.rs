use std::ptr::copy_nonoverlapping;

use crate::{
    parser::{Execution, Expr, Stmt},
    vm::{
        bytecode::OpCode,
        runtime::Heap,
        value::{TypeId, Value},
    },
};

pub fn compile_stmt(stmt: Stmt, code: &mut Vec<OpCode>, heap: &mut Heap) {
    match stmt {
        Stmt::Exe(e) => compile_execution(e, code, heap),
        Stmt::Assignment { dst, rhs } => {
            // compile_execution(lhs, rt);
            compile_execution(rhs, code, heap);
            // TODO!!!!!!!
            // match on lhs if singular we can avoid execution
            code.push(OpCode::SetGlobal(dst.id));
        }
    };
}

pub fn compile_execution(exe: Execution, code: &mut Vec<OpCode>, heap: &mut Heap) {
    match exe {
        Execution::Single(e) => compile_expr(e, code, heap),
        Execution::Called(instance, message, args) => {
            let message = match message {
                Expr::Ident(i) => i,
                Expr::Lit(_) => panic!("Cannot have literal as message"),
                Expr::Block(_) => panic!("Cannot have block as message"),
            };
            compile_expr(instance, code, heap);

            let arg_count = args.len() as u64;
            for arg in args {
                compile_expr(arg, code, heap);
            }

            code.push(OpCode::SendMessage {
                id: message.id,
                arg_count,
            });
        }
    }
}

pub fn compile_expr(expr: Expr, code: &mut Vec<OpCode>, heap: &mut Heap) {
    match expr {
        Expr::Lit(l) => code.push(OpCode::PushLit(Value::from_literal(l, heap))),
        Expr::Ident(i) => code.push(OpCode::PushGlobal(i.id)),
        Expr::Block(b) => {
            let mut ops = Vec::new();
            for stmt in b {
                compile_stmt(stmt, &mut ops, heap);
            }

            let header_size = 16;
            let ptr = heap
                .alloc(
                    (header_size + ops.len() * size_of::<OpCode>()) as u64,
                    TypeId::CompiledBytecode,
                )
                .unwrap();
            unsafe {
                ptr.cast().write(ops.len() as u64);
                let slice_start = ptr.add(header_size).cast::<OpCode>().as_ptr();
                copy_nonoverlapping(ops.as_ptr(), slice_start, ops.len());
            }
            code.push(OpCode::PushLit(Value::from_ptr(ptr.as_ptr() as u64)));
        }
    }
}
