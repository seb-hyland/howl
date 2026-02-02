use crate::vm::{
    runtime::{HeapMap, Runtime},
    value::Value,
};
use std::{mem, ptr::NonNull};

type ExternHandler = fn(rt: &mut Runtime, arg_count: u64) -> Option<Value>;
pub enum Handler {
    Extern(ExternHandler),
}

#[repr(u128)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    PushLit(Value),
    PushGlobal(u64),
    SetGlobal(u64),
    SendMessage { id: u64, arg_count: u64 },
}

pub fn flush_runtime(rt: &mut Runtime) {
    let code = mem::take(&mut rt.code);
    println!("Compiled bytecode: {code:?}");
    exe(code, rt);
}

pub fn exe<I: IntoIterator<Item = OpCode>>(code: I, rt: &mut Runtime) {
    for op in code {
        match op {
            OpCode::PushLit(v) => rt.push_stack(v),
            OpCode::PushGlobal(g) => {
                let global_val = rt.globals.vars.get(&Value::from_uint(g));
                if let Some(v) = global_val {
                    rt.push_stack(v);
                } else {
                    panic!(
                        "Variable {g} doesn't exist in globals: {:?}",
                        rt.globals.idents
                    );
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
                    .get(&Value::from_uint(type_id as u64))
                    .unwrap_or_else(|| panic!("Type {} doesn't exist", type_id as u64));
                let ty_map = unsafe {
                    HeapMap::from_ptr(
                        NonNull::new(ty.as_ptr() as *mut Value).unwrap(),
                        &mut rt.heap,
                    )
                };
                let handler = ty_map.get(&Value::from_uint(id)).unwrap_or_else(|| {
                    panic!(
                        "Failed to get handler with id {id}: {:#?}",
                        rt.globals.idents
                    )
                });
                let res = unsafe {
                    std::mem::transmute::<usize, ExternHandler>(handler.as_uint() as usize)
                }(rt, arg_count);
                if let Some(output) = res {
                    rt.push_stack(output);
                }
            }
        }
    }
}
