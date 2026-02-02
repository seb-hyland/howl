use crate::vm::{
    runtime::Runtime,
    value::{TypeId, Value},
};

pub fn define_int(rt: &mut Runtime) {
    let id = TypeId::Int;
    rt.define_type(id);

    define_int_add(rt, id);
    define_int_sumall(rt, id);
    define_int_display(rt, id);
    define_int_mul(rt, id);
    define_int_eq(rt, id);
    define_int_neq(rt, id);
}

fn define_int_add(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 1 {
            panic!("Bad args!")
        }
        let rhs = rt.pop_stack().as_int();
        let lhs = rt.pop_stack().as_int();
        Some(Value::from_int(lhs + rhs))
    }
    rt.register_handler("+", handler, id);
}

fn define_int_eq(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 1 {
            panic!("Bad args!")
        }
        let rhs = rt.pop_stack().as_int();
        let lhs = rt.pop_stack().as_int();
        Some(Value::from_bool(lhs == rhs))
    }
    rt.register_handler("==", handler, id);
}

fn define_int_neq(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 1 {
            panic!("Bad args!")
        }
        let rhs = rt.pop_stack().as_int();
        let lhs = rt.pop_stack().as_int();
        Some(Value::from_bool(lhs != rhs))
    }
    rt.register_handler("!=", handler, id);
}

fn define_int_sumall(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        let mut sum = 0;
        for _ in 0..arg_count {
            sum += rt.pop_stack().as_int();
        }
        sum += rt.pop_stack().as_int();
        Some(Value::from_int(sum))
    }
    rt.register_handler("+allof", handler, id);
}

fn define_int_mul(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 1 {
            panic!("Bad args!")
        }
        let rhs = rt.pop_stack().as_int();
        let lhs = rt.pop_stack().as_int();
        Some(Value::from_int(lhs * rhs))
    }
    rt.register_handler("*", handler, id);
}

fn define_int_display(rt: &mut Runtime, id: TypeId) {
    fn handler(rt: &mut Runtime, arg_count: u64) -> Option<Value> {
        if arg_count != 0 {
            panic!("Bad args!")
        }
        let lhs = rt.pop_stack().as_int();
        println!("(Int) {lhs}");
        None
    }
    rt.register_handler("display", handler, id);
}
