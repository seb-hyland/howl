use crate::vm::runtime::Runtime;

pub mod int;

pub fn define_std_types(rt: &mut Runtime) {
    int::define_int(rt);
}
