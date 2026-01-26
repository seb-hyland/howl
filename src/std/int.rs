use std::collections::HashMap;

use crate::vm::{
    globals::{Runtime, Type},
    value::Value,
};

pub fn define_int(rt: &mut Runtime) {
    let mut int = Type {
        instance_fields: Vec::new(),
        type_fields: Vec::new(),
        handlers: HashMap::new(),
    };
    define_int_add(rt, &mut int);
    define_int_sumall(rt, &mut int);
    define_int_display(rt, &mut int);
    define_int_mul(rt, &mut int);
    rt.define_type(1, int);
}

fn define_int_add(rt: &mut Runtime, int: &mut Type) {
    let handler = |rt: &mut Runtime, arg_count| -> Option<Value> {
        if arg_count != 1 {
            panic!("Bad args!")
        }
        let rhs = rt.pop_stack().as_int();
        let lhs = rt.pop_stack().as_int();
        Some(Value::from_int(lhs + rhs))
    };
    rt.register_handler("+", handler, int);
}

fn define_int_sumall(rt: &mut Runtime, int: &mut Type) {
    let handler = |rt: &mut Runtime, arg_count| -> Option<Value> {
        let mut sum = 0;
        for _ in 0..arg_count {
            sum += rt.pop_stack().as_int();
        }
        sum += rt.pop_stack().as_int();
        Some(Value::from_int(sum))
    };
    rt.register_handler("+allof", handler, int);
}

fn define_int_mul(rt: &mut Runtime, int: &mut Type) {
    let handler = |rt: &mut Runtime, arg_count| -> Option<Value> {
        if arg_count != 1 {
            panic!("Bad args!")
        }
        let rhs = rt.pop_stack().as_int();
        let lhs = rt.pop_stack().as_int();
        Some(Value::from_int(lhs * rhs))
    };
    rt.register_handler("*", handler, int);
}

fn define_int_display(rt: &mut Runtime, int: &mut Type) {
    let handler = |rt: &mut Runtime, arg_count| -> Option<Value> {
        if arg_count != 0 {
            panic!("Bad args!")
        }
        let lhs = rt.pop_stack().as_int();
        println!("(Int) {lhs}");
        None
    };
    rt.register_handler("display", handler, int);
}
