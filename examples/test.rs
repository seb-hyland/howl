use howl::{eval, vm::globals::Runtime};

fn main() {
    let mut rt = Runtime::new();
    eval(include_str!("test.howl"), &mut rt);

    println!("\nBytecode: {:X?}", &rt.code);
    println!("\nGlobals: {:#X?}", rt.globals.vars);
    println!("\nIdents: {:#X?}", rt.globals.idents);
}
