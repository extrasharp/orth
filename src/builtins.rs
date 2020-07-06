use crate::{
    Builtin,
    Context,
    Value,
};

//

pub static BUILTINS: &[Builtin] = &[
    Builtin {
        name: "@",
        f: | ctx: &mut Context | {
            let sym = ctx.stack.pop().unwrap();
            let val = ctx.stack.pop().unwrap();

            if let Value::Symbol(sym) = sym {
                ctx.env.insert(&sym.name, val);
            }
        },
    },

    Builtin {
        name: "show-ctx",
        f: | ctx: &mut Context | {
            println!("{:?}", ctx);
        },
    },

    Builtin {
        name: "show-top",
        f: | ctx: &mut Context | {
            println!("{:?}", ctx.stack.peek());
        },
    },

    Builtin {
        name: "show-stack",
        f: | ctx: &mut Context | {
            let len = ctx.stack.stk.len();
            println!("stack:");
            for (i, val) in ctx.stack.stk.iter().enumerate().rev() {
                println!(" {}: {:?}", len - i - 1, val);
            }
        },
    },

    Builtin {
        name: "show-env",
        f: | ctx: &mut Context | {
            println!("env:");
            for (key, val) in &ctx.env.table {
                match val {
                    Value::Quotation(_) => println!(" {}: quotation", key),
                    _ => println!(" {}: {:?}", key, val),
                }
            }
        },
    },

    // basics

    Builtin {
        name: "swap",
        f: | ctx: &mut Context | {
            let v1 = ctx.stack.pop().unwrap();
            let v2 = ctx.stack.pop().unwrap();
            ctx.stack.push(v1);
            ctx.stack.push(v2);
        },
    },

    // vectors

    Builtin {
        name: "make-vec",
        f: | ctx: &mut Context | {
            ctx.stack.push(Value::Vec(Vec::new()))
        },
    },

    Builtin {
        name: "vpush!",
        f: | ctx: &mut Context | {
            let val = ctx.stack.pop().unwrap();
            let mut vec = ctx.stack.pop().unwrap();

            if let Value::Vec(r_vec) = &mut vec {
                r_vec.push(val);
            }

            ctx.stack.push(vec);
        },
    },

    Builtin {
        name: "vget",
        f: | ctx: &mut Context | {
            let at = ctx.stack.pop().unwrap();
            let mut vec = ctx.stack.pop().unwrap();

            if let Value::Int(at) = at {
                if let Value::Vec(r_vec) = &mut vec {
                    ctx.stack.push(r_vec.get(at as usize).unwrap().clone());
                }
            }
        },
    },
];

