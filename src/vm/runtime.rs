use crate::vm::globals::{Handler, OpCode, Runtime};

fn runtime(rt: &mut Runtime) {
    let pc = rt.pc;

    loop {
        if pc >= rt.code.len() {
            break;
        }
        match rt.code[pc] {
            OpCode::PushLiteral(v) => rt.push_stack(v),
            OpCode::PushGlobal(g) => {
                let global_val = rt.globals.vars[g];
                if let Some(v) = global_val {
                    rt.push_stack(v);
                } else {
                    panic!("Variable doesn't exist in globals");
                }
            }
            OpCode::SetGlobal(g) => {
                let stack_value = rt.pop_stack();
                rt.globals.vars[g] = Some(stack_value);
            }
            OpCode::SendMessage { id, arg_count } => {
                let type_id = rt.peek().type_of();
                let ty = rt.globals.types.get(&type_id).expect("Type doesn't exist");
                let handler = ty.handlers.get(&id).expect("Handler doesn't exist");
                match handler {
                    Handler::Extern(lambda) => {
                        let res = lambda(rt, arg_count);
                        if let Some(output) = res {
                            rt.push_stack(output);
                        }
                    }
                }
            }
        }
        rt.pc += 1;
    }
}
