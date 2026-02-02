use crate::vm::{
    bytecode::{OpCode, exe},
    runtime::Runtime,
    value::{TypeId, Value},
};
use std::slice;

pub fn define_block(rt: &mut Runtime) {
    let id = TypeId::CompiledBytecode;
    rt.define_type(id);

    define_block_run(rt, id);
    define_block_loop(rt, id);
}

fn define_block_run(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 0 {
            panic!("Bad args!")
        }
        let ptr = rt.pop_stack().as_ptr() as *const u8;
        unsafe { run_block(ptr, rt) };

        None
    }
    rt.register_handler("value", handler, id);
}

pub unsafe fn run_block(ptr: *const u8, rt: &mut Runtime) {
    let header_size = 16;

    let ops = unsafe {
        let ops_len = ptr.cast::<u64>().read();
        let slice_start = ptr.add(header_size).cast::<OpCode>();
        slice::from_raw_parts(slice_start, ops_len as usize)
    };
    exe(ops.iter().copied(), rt);
}

fn define_block_loop(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 0 {
            panic!("Bad args!")
        }
        let ptr = rt.pop_stack().as_ptr() as *const u8;
        loop {
            unsafe { run_block(ptr, rt) };
        }
    }
    rt.register_handler("loop", handler, id);
}
