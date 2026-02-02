use crate::vm::runtime::Runtime;

pub mod block;
pub mod bool;
pub mod int;
pub mod string;

pub fn define_std_types(rt: &mut Runtime) {
    int::define_int(rt);
    block::define_block(rt);
    bool::define_bool(rt);
    string::define_string(rt);
}
