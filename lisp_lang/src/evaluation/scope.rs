use lazy_static::lazy_static;

use crate::parsing::LispVal;

#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    pub context: String,
    pub bindings: im::HashMap<String, LispVal>,
}

impl Scope {
    pub fn empty(context: String) -> Scope {
        Scope {
            context,
            bindings: im::HashMap::<String, LispVal>::new(),
        }
    }

    pub fn default() -> Scope {
        INITIAL_SCOPE.clone()
    }

    pub fn with_context(&self, context: String) -> Scope {
        Scope {
            context,
            bindings: self.bindings.clone(),
        }
    }

    pub fn bind(&self, name: String, value: LispVal) -> Scope {
        Scope {
            context: self.context.clone(),
            bindings: self.bindings.update(name, value),
        }
    }

    pub fn get(&self, name: &str) -> Option<&LispVal> {
        self.bindings.get(name)
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

pub const MAIN_CONTEXT: &str = "main";

#[macro_export]
macro_rules! lisp_scope {
    // Base case
    () => {
        Scope::empty(MAIN_CONTEXT.to_string())
    };
    // Recursive case
    ($name:ident = $value:expr, $($rest:tt)*) => {
        lisp_scope!($($rest)*).bind(stringify!($name).to_string(), $value)
    };
}

lazy_static! {
    pub static ref INITIAL_SCOPE: Scope = lisp_scope!{
        MIN_INT = LispVal::Number(i64::MIN),
        MAX_INT = LispVal::Number(i64::MAX),
    };
}


