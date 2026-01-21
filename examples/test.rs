use howl::{eval, vm::globals::Runtime};

fn main() {
    let mut rt = Runtime::new();
    eval(include_str!("test.howl"), &mut rt);

    println!("\nBytecode: {:?}", &rt.code);
    println!("\nGlobals: {:#X?}", rt.globals.vars);
}
