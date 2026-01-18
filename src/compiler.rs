use crate::{
    parser::{Execution, Expr, Stmt},
    vm::{
        globals::{OpCode, Runtime, Type},
        value::Value,
    },
};
use std::collections::HashMap;

pub fn compile_stmt(stmt: Stmt, rt: &mut Runtime) {
    match stmt {
        Stmt::TypeDefinition {
            name,
            instance_fields,
            type_fields,
        } => {
            rt.define_type(
                name.id,
                Type {
                    instance_fields,
                    type_fields,
                    handlers: HashMap::new(),
                },
            );
        }
        Stmt::Exe(e) => compile_execution(e, rt),
        Stmt::Assignment { lhs, rhs } => {
            // compile_execution(lhs, rt);
            compile_execution(rhs, rt);
            let dest = match lhs {
                Execution::Single(e) => match e {
                    Expr::Ident(i) => i,
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            };
            // TODO!!!!!!!
            // match on lhs if singular we can avoid execution
            rt.push_op(OpCode::PushGlobal(dest.id));
        }
    };
}

pub fn compile_execution(exe: Execution, rt: &mut Runtime) {
    match exe {
        Execution::Single(e) => compile_expr(e, rt),
        Execution::Called {
            instance,
            message,
            args,
        } => {
            compile_expr(instance, rt);

            let arg_count = args.len();
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
        Expr::Literal(l) => rt.push_op(OpCode::PushLiteral(Value::from_literal(l))),
        Expr::Ident(i) => rt.push_op(OpCode::PushGlobal(i.id)),
        Expr::Tuple(t) => todo!(),
    }
}
