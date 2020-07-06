use std::{
    collections::HashMap,
    fs,
};

use orth::{
    self,
    builtins,
    Context,
};

//

fn main() {
    let mut builtins = HashMap::new();
    for b in builtins::BUILTINS {
        builtins.insert(b.name, *b);
    }

    let file = fs::read_to_string("test.orth").unwrap();
    let tokens = orth::tokenize(&file).unwrap();
    let values = orth::parse(&tokens, &builtins);
    // println!("{:?}", values);

    let mut ctx = Context::new();
    orth::eval(&values.unwrap(), &mut ctx);
}
