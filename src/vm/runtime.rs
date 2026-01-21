use crate::vm::globals::{Handler, OpCode, Runtime};

pub fn exe(rt: &mut Runtime) {
    rt.globals.vars.resize(rt.globals.idents.len(), None);
    loop {
        let pc = rt.pc;

        if pc >= rt.code.len() {
            break;
        }
        match rt.code[pc] {
            OpCode::PushLit(v) => rt.push_stack(v),
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
                let type_id = rt.peek_at(arg_count).type_of();
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
