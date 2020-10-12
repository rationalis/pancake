pub use inlinable_string::InlinableString;
use std::collections::HashMap;
//use crate::types::Atom;
use crate::ast::Expr;

pub const SPECIAL_IDENTS: [&str; 6] = ["call", "let", "fn", "true", "false", "not"];
pub type Stack = Vec<Expr>;
pub type Identifier = InlinableString;

#[derive(Clone, Debug)]
// pub struct Context(HashMap<Identifier, Expr>);
pub struct Context<T: Clone> {
    map: HashMap<Identifier, T>,
    stack: Vec<Identifier>,
    counter: u32,
}

impl<T: Clone> Context<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::with_capacity(16),
            stack: Vec::with_capacity(16),
            counter: 0
        }
    }

    pub fn get(&self, ident: &str) -> Option<&T> {
        self.map.get(ident)
    }

    pub fn insert(&mut self, ident: &str, elem: T) -> Option<T> {
        // use crate::ops;
        // if SPECIAL_IDENTS.contains(&ident)
        //     || ops::get_boolean_op(ident).is_some()
        //     || ops::get_stack_op(ident).is_some()
        // {
        //     panic!("Attempted to rebind reserved word {}.", ident);
        // }

        self.map.insert(Identifier::from(ident), elem)
        // if self.map.insert(InlinableString::from(ident), atom).is_some() {
        //     panic!("Attempted to rebind existing variable {}.", ident);
        // }
    }

    pub fn push(&mut self, elem: T) {
        let new_id = Identifier::from(format!("_{}", self.counter));
        self.map.insert(new_id.clone(), elem);
        self.stack.push(new_id);
        self.counter += 1;
    }

    pub fn pop(&mut self) -> &T {
        self.map.get(&self.stack.pop().unwrap()).unwrap()
    }

    pub fn in_child_scope<R>(&self, cb: impl FnOnce(&mut Self) -> R) -> R {
        // let temp = self.map.clone();
        // let res = cb(self);
        // self.map = temp;
        // res
        cb(&mut self.clone())
    }

    pub fn with_binding<R>(&mut self, ident: &str, elem: T,
                           cb: impl FnOnce(&mut Self) -> R) -> R {
        let temp = self.insert(ident, elem);
        let res = cb(self);
        if let Some(prev) = temp {
            self.insert(ident, prev);
        }
        res
    }
}



use rusty_v8 as v8;

#[test]
fn test_v8() {
    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let code = v8::String::new(scope, "'Hello' + ' World!'").unwrap();
    println!("javascript code: {}", code.to_rust_string_lossy(scope));

    let mut script = v8::Script::compile(scope, code, None).unwrap();
    let result = script.run(scope).unwrap();
    let result = result.to_string(scope).unwrap();
    println!("result: {}", result.to_rust_string_lossy(scope));
}
