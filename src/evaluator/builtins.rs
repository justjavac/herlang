use evaluator::object::*;
use std::collections::HashMap;

pub fn new_builtins() -> HashMap<String, Object> {
    let mut builtins = HashMap::new();
    // Monkey builtins
    builtins.insert(String::from("len"), Object::Builtin(1, monkey_len));
    builtins.insert(String::from("first"), Object::Builtin(1, monkey_first));
    builtins.insert(String::from("last"), Object::Builtin(1, monkey_last));
    builtins.insert(String::from("rest"), Object::Builtin(1, monkey_rest));
    builtins.insert(String::from("push"), Object::Builtin(2, monkey_push));
    builtins.insert(String::from("puts"), Object::Builtin(-1, her_output));

    // herlang builtin, but not aba-aba
    builtins.insert(String::from("quit"), Object::Builtin(-1, her_quit));
    builtins.insert(String::from("print"), Object::Builtin(1, her_print));
    builtins.insert(String::from("repr"), Object::Builtin(1, her_repr));
    builtins.insert(String::from("str"), Object::Builtin(1, her_str));
    builtins.insert(String::from("atoi"), Object::Builtin(1, her_atoi));

    // Aba-aba builtins
    builtins.insert(String::from("哼"), Object::Builtin(-1, her_quit));
    builtins.insert(String::from("小作文"), Object::Builtin(-1, her_output));
    builtins.insert(String::from("聚焦"), Object::Builtin(1, her_print));
    builtins.insert(String::from("复用"), Object::Builtin(1, her_repr));
    builtins.insert(String::from("疏通"), Object::Builtin(1, her_str));
    builtins.insert(String::from("抹零"), Object::Builtin(1, her_atoi));
    builtins
}

fn monkey_len(args: Vec<Object>) -> Object {
    match &args[0] {
        Object::String(s) => Object::Int(s.len() as i64),
        Object::Array(o) => Object::Int(o.len() as i64),
        o => Object::Error(format!("argument to `len` not supported, got {o}")),
    }
}

fn monkey_first(args: Vec<Object>) -> Object {
    match &args[0] {
        Object::Array(o) => {
            if let Some(ao) = o.first() {
                ao.clone()
            } else {
                Object::Null
            }
        }
        o => Object::Error(format!("argument to `first` must be array. got {o}")),
    }
}

fn monkey_last(args: Vec<Object>) -> Object {
    match &args[0] {
        Object::Array(o) => {
            if let Some(ao) = o.last() {
                ao.clone()
            } else {
                Object::Null
            }
        }
        o => Object::Error(format!("argument to `last` must be array. got {o}")),
    }
}

fn monkey_rest(args: Vec<Object>) -> Object {
    match &args[0] {
        Object::Array(o) => {
            if !o.is_empty() {
                Object::Array(o[1..].to_vec())
            } else {
                Object::Null
            }
        }
        o => Object::Error(format!("argument to `rest` must be array. got {o}")),
    }
}

fn monkey_push(args: Vec<Object>) -> Object {
    match &args[0] {
        Object::Array(o) => {
            let mut arr = o.clone();
            arr.push(args[1].clone());
            Object::Array(arr)
        }
        o => Object::Error(format!("argument to `push` must be array. got {o}")),
    }
}

fn her_str(args: Vec<Object>) -> Object {
    match &args[0] {
        Object::String(s) => Object::String(s.to_string()),
        x => Object::String(format!("{x}")),
    }
}

fn her_repr(args: Vec<Object>) -> Object {
    Object::String(format!("{}", args[0]))
}

fn her_print(args: Vec<Object>) -> Object {
    match &args[0] {
        Object::String(ref o) => {
            println!("{o}");
            Object::Null
        }
        o => Object::Error(format!("argument to `push` must be array. got {o}")),
    }
}

fn her_output(args: Vec<Object>) -> Object {
    for arg in args {
        println!("{arg}");
    }
    Object::Null
}

fn her_quit(args: Vec<Object>) -> Object {
    match args.len() {
        0 => std::process::exit(0),
        1 => match &args[0] {
            Object::Int(i) => std::process::exit(*i as i32),
            o => Object::Error(format!("argument to `quit` must be int. got {o}")),
        },
        _ => Object::Error(format!(
            "Too many arguments to `quit` (want 0 or 1, got {})",
            args.len()
        )),
    }
}

fn her_atoi(args: Vec<Object>) -> Object {
    match &args[..] {
        [Object::String(s)] => s.parse().map(Object::Int).unwrap_or_else(|_| {
            Object::Error(format!(
                "argument to `atoi` must be valid digits. got {s:?}"
            ))
        }),
        _ => Object::Error(format!(
            "illegal argument to `atoi` (want 1 string, got {args:?}"
        )),
    }
}
