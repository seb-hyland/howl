use std::ptr::NonNull;

use crate::vm::{
    runtime::{HeapMap, Runtime},
    value::Value,
};

type ExternHandler = fn(rt: &mut Runtime, arg_count: u64) -> Option<Value>;
pub enum Handler {
    Extern(ExternHandler),
}

#[derive(Debug)]
pub enum OpCode {
    PushLit(Value),
    PushGlobal(u64),
    SetGlobal(u64),
    SendMessage { id: u64, arg_count: u64 },
}

pub fn exe(rt: &mut Runtime) {
    loop {
        let pc = rt.pc as usize;

        if pc >= rt.code.len() {
            break;
        }
        match rt.code[pc] {
            OpCode::PushLit(v) => rt.push_stack(v),
            OpCode::PushGlobal(g) => {
                let global_val = rt.globals.vars.get(&Value::from_uint(g));
                if let Some(v) = global_val {
                    rt.push_stack(v);
                } else {
                    panic!("Variable doesn't exist in globals");
                }
            }
            OpCode::SetGlobal(g) => {
                let stack_value = rt.pop_stack();
                rt.globals.vars.insert(Value::from_uint(g), stack_value);
            }
            OpCode::SendMessage { id, arg_count } => {
                let type_id = rt.peek_at(arg_count).type_of();
                let ty = rt
                    .globals
                    .types
                    .get(&Value::from_uint(type_id))
                    .expect("Type doesn't exist");
                let ty_map = unsafe {
                    HeapMap::from_ptr(
                        NonNull::new(ty.as_ptr() as *mut Value).unwrap(),
                        &mut rt.heap,
                    )
                };
                let handler = ty_map
                    .get(&Value::from_uint(id))
                    .expect("Handler doesn't exist");
                let res = unsafe { std::mem::transmute::<u64, ExternHandler>(handler.as_uint()) }(
                    rt, arg_count,
                );
                if let Some(output) = res {
                    rt.push_stack(output);
                }
            }
        }
        rt.pc += 1;
    }
}
