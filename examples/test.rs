use howl::vm::globals::Runtime;

fn main() {
    let mut rt = Runtime::new();
    let file = include_str!("test.howl");
    let syntax = howl::parse(file, &mut rt).unwrap();
    // println!("{:#?}", syntax);
    howl::run(syntax, &mut rt);

    println!("\nBytecode: {:#X?}", &rt.code);
    println!("\nGlobals: {:#X?}", rt.globals.vars);
    println!("\nIdents: {:#X?}", rt.globals.idents);
}
