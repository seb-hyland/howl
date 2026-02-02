use crate::{
    std::block::run_block,
    vm::{
        runtime::Runtime,
        value::{TypeId, Value},
    },
};

pub fn define_bool(rt: &mut Runtime) {
    let true_id = TypeId::True;
    let false_id = TypeId::False;

    rt.define_type(true_id);
    rt.define_type(false_id);

    define_if::<true>(rt, true_id);
    define_if::<false>(rt, false_id);
}

fn define_if<const BOOLEAN: bool>(rt: &mut Runtime, id: TypeId) {
    fn handler<const BOOLEAN: bool>(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 1 && arg_count != 2 {
            panic!("Expected 1 or 2 arguments, got {}", arg_count);
        }

        if arg_count == 1 {
            let arg1 = rt.pop_stack().as_ptr() as *const u8;
            if BOOLEAN {
                unsafe { run_block(arg1, rt) };
            }
        } else {
            let arg2 = rt.pop_stack().as_ptr() as *const u8;
            let arg1 = rt.pop_stack().as_ptr() as *const u8;

            if BOOLEAN {
                unsafe { run_block(arg1, rt) };
            } else {
                unsafe { run_block(arg2, rt) };
            }
        }

        None
    }
    rt.register_handler("ifTrue", handler::<BOOLEAN>, id);
}
