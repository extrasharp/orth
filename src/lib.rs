use std::{
    collections::HashMap,
    fmt,
    rc::Rc,
};

//

pub mod builtins;

//

#[derive(Debug)]
pub enum Error {
    UnfinishedString,
    InvalidSymbol,
    InvalidWord,
}

#[derive(Debug)]
pub enum Token<'a> {
    Symbol(&'a str),
    String(&'a str),
    Word(&'a str),
}

fn char_is_delimiter(ch: char) -> bool {
    char::is_whitespace(ch) ||
    ch == ';'
}

fn char_is_word_valid(ch: char) -> bool {
    ch != '"' &&
    ch != ':' &&
    ch != '{' &&
    ch != '}'
}

fn char_is_symbol_valid(ch: char) -> bool {
    ch != '"' &&
    ch != ':'
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, (usize, Error)> {
    enum State {
        Empty,
        InWord,
        InSymbol,
        InComment,
        InString,
    }

    let mut state = State::Empty;

    let mut start = 0;
    let mut end = 0;

    let mut ret = Vec::new();

    let mut line_num = 1;

    for (i, ch) in input.chars().enumerate() {
        // TODO where to put this so line num correctly reports errors
        //  i.e. ":\n" invalid symbol
        if ch == '\n' {
            line_num += 1;
        }

        match state {
            State::Empty => {
                if char::is_whitespace(ch) {
                    continue;
                }

                state = match ch {
                    ':' => State::InSymbol,
                    ';' => State::InComment,
                    '"' => State::InString,
                    _   => State::InWord,
                };

                start = i;
                end = start;
            },

            State::InWord => {
                // error if word contains " or :
                if char_is_delimiter(ch) {
                    ret.push(Token::Word(&input[start..=end]));
                    state = State::Empty;
                    continue;
                } else if !char_is_word_valid(ch) {
                    return Err((line_num, Error::InvalidWord));
                }

                end += 1;
            },

            // todo make sure cant parse symbol as number
            State::InSymbol => {
                if char_is_delimiter(ch) {
                    if start == end {
                        return Err((line_num, Error::InvalidSymbol));
                    }
                    ret.push(Token::Symbol(&input[start..=end]));
                    state = State::Empty;
                    continue;
                } else if !char_is_symbol_valid(ch) {
                    return Err((line_num, Error::InvalidSymbol));
                }

                end += 1;
            },

            State::InComment => {
                if ch == '\n' {
                    state = State::Empty;
                }
            },

            State::InString => {
                if ch == '"' {
                    ret.push(Token::String(&input[(start+1)..=end]));
                    state = State::Empty;
                    continue;
                } else if ch == '\n' {
                    return Err((line_num, Error::UnfinishedString));
                }

                end += 1;
            },
        }
    }

    Ok(ret)
}

//

#[derive(Copy, Clone)]
pub struct Builtin {
    pub name: &'static str,
    pub f: fn(&mut Context),
}

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Builtin")
         .field("name", &self.name)
         .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub id: u64,
    pub name: Rc<String>,
}

// TODO make quote open and quote close specific types of values
//      rather than builtins
#[derive(Clone, Debug)]
pub enum Value {
    Symbol(Symbol),
    Int(i64),
    Float(f64),
    Boolean(bool),
    Map(HashMap<Value, Value>),
    Vec(Vec<Value>),
    String(String),
    QuoteOpen,
    QuoteClose,
    Quotation(Vec<Value>),
    Builtin(Builtin),
    Word(String), // todo word_table ? or cache
}

// parse everything into values
// for now just put in { and } as builtins
// send in parsing env
pub fn parse(tokens: &[Token], builtins: &HashMap<&str, Builtin>) -> Result<Vec<Value>, Error> {
    let mut ret = Vec::new();

    let mut symbol_table = HashMap::<&str, Symbol>::new();
    let mut symbols_ct = 0;

    for token in tokens {
        match token {
            Token::Symbol(s) => {
                match symbol_table.get(s) {
                    Some(sym) => { 
                        ret.push(Value::Symbol(sym.clone()));
                    },
                    None => {
                        let sym = Symbol { id: symbols_ct, name: Rc::new(String::from(&(*s)[1..])) };
                        // dont cone here? idk
                        symbol_table.insert(s, sym.clone());
                        ret.push(Value::Symbol(sym));
                        symbols_ct += 1;
                    }
                }
            },
            Token::String(s) => {
                ret.push(Value::String(String::from(*s)));
            },
            Token::Word(s) => {
                if let Ok(val) = s.parse() {
                    ret.push(Value::Int(val));
                } else if let Ok(val) = s.parse() {
                    ret.push(Value::Float(val));
                } else if s == &"#t" {
                    ret.push(Value::Boolean(true));
                } else if s == &"#f" {
                    ret.push(Value::Boolean(false));
                } else if s == &"{" {
                    ret.push(Value::QuoteOpen);
                } else if s == &"}" {
                    ret.push(Value::QuoteClose);
                } else if let Some(b) = builtins.get(s) {
                    ret.push(Value::Builtin(*b));
                } else {
                    ret.push(Value::Word(String::from(*s)));
                }
            },
        }
    }

    Ok(ret)
}

//

// evaluate a vec of values with stack and env and stuff
pub fn eval(values: &[Value], ctx: &mut Context) {
    let mut quotation_level = 0;
    let mut quotation_buf = Vec::new();

    for value in values {
        if let Value::QuoteOpen = value {
            quotation_level += 1;
            if quotation_level == 1 {
                quotation_buf.clear();
                continue;
            }
        } else if let Value::QuoteClose = value {
            quotation_level -= 1;
            if quotation_level == 0 {
                ctx.stack.push(Value::Quotation(quotation_buf.clone()));
            }
        }

        if quotation_level > 0 {
            quotation_buf.push(value.clone());
            continue;
        }

        match value {
            Value::Builtin(b) => {
                (b.f)(ctx);
            },

            Value::Word(w) => {
                let v = ctx.env.get(&w).cloned();
                match v {
                    Some(val) => {
                        match val {
                            Value::Quotation(vec) => {
                                eval(&vec, ctx);
                            },
                            Value::Builtin(b) => {
                                (b.f)(ctx);
                            },
                            _ => {
                                ctx.stack.push(val.clone());
                            }
                        }
                    },
                    None => {
                        // error
                        println!("{} not found", w);
                    }
                }
            },

            Value::QuoteOpen |
            Value::QuoteClose => {}

            _ => {
                // dont clone here idk
                ctx.stack.push(value.clone());
            },

        }
    }
}

//

#[derive(Debug)]
pub struct Env {
    pub table: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        let table = HashMap::new();
        Self {
            table,
        }
    }

    pub fn insert(&mut self, s: &str, val: Value) {
        self.table.insert(String::from(s), val);
    }

    pub fn get(&self, s: &str) -> Option<&Value> {
        self.table.get(s)
    }

    // pub fn get_mut(&mut self, s: &str) -> Option<&mut Value> {
        // self.table.get_mut(s)
    // }
}

#[derive(Debug)]
pub struct Stack {
    pub stk: Vec<Value>
}

impl Stack {
    pub fn new() -> Self {
        let stk = Vec::new();
        Self {
            stk,
        }
    }

    pub fn push(&mut self, val: Value) {
        self.stk.push(val);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stk.pop()
    }

    pub fn peek(&self) -> Option<&Value> {
        self.stk.last()
    }
}

#[derive(Debug)]
pub struct Context {
    env: Env,
    stack: Stack,
    quotation_level: u8,
}

impl Context {
    pub fn new() -> Self {
        Self {
            env: Env::new(),
            stack: Stack::new(),
            quotation_level: 0,
        }
    }
}
