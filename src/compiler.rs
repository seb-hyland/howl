use crate::{
    parser::{Execution, Expr, Stmt},
    vm::{
        globals::{OpCode, Runtime},
        value::Value,
    },
};

pub fn compile_stmt(stmt: Stmt, rt: &mut Runtime) {
    match stmt {
        Stmt::Exe(e) => compile_execution(e, rt),
        Stmt::Assignment { dst, rhs } => {
            // compile_execution(lhs, rt);
            compile_execution(rhs, rt);
            // TODO!!!!!!!
            // match on lhs if singular we can avoid execution
            rt.push_op(OpCode::SetGlobal(dst.id));
        }
    };
}

pub fn compile_execution(exe: Execution, rt: &mut Runtime) {
    match exe {
        Execution::Single(e) => compile_expr(e, rt),
        Execution::Called(instance, message, args) => {
            let message = match message {
                Expr::Ident(i) => i,
                Expr::Lit(_) => panic!("Cannot have literal as message"),
            };
            compile_expr(instance, rt);

            let arg_count = args.len() as u64;
            for arg in args {
                compile_expr(arg, rt);
            }

            rt.push_op(OpCode::SendMessage {
                id: message.id,
                arg_count,
            });
        }
    }
}

pub fn compile_expr(expr: Expr, rt: &mut Runtime) {
    match expr {
        Expr::Lit(l) => rt.push_op(OpCode::PushLit(Value::from_literal(l))),
        Expr::Ident(i) => rt.push_op(OpCode::PushGlobal(i.id)),
    }
}
