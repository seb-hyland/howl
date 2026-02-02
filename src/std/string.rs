use std::{slice, str};

use crate::vm::{
    runtime::Runtime,
    value::{TypeId, Value},
};

pub fn define_string(rt: &mut Runtime) {
    let id = TypeId::String;

    rt.define_type(id);

    define_string_display(rt, id);
    define_string_output(rt, id);
}

pub fn as_string(v: Value) -> &'static str {
    let ptr = v.as_ptr() as *const u8;
    unsafe {
        let len = ptr.cast::<u64>().read();
        str::from_utf8_unchecked(slice::from_raw_parts(ptr.add(16), len as usize))
    }
}

fn define_string_display(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 0 {
            panic!("Bad args!")
        }

        let lhs = rt.pop_stack();
        let s = as_string(lhs);
        println!("(String) {s}");

        None
    }
    rt.register_handler("display", handler, id);
}

fn define_string_output(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 0 {
            panic!("Bad args!")
        }

        let lhs = rt.pop_stack();
        let s = as_string(lhs);
        println!("{s}");

        None
    }
    rt.register_handler(">>", handler, id);
}
