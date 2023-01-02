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

lazy_static! {
    pub static ref INITIAL_SCOPE: Scope = {
        let scope = Scope::empty(MAIN_CONTEXT.to_string())
            .bind("MIN_INT".to_string(), LispVal::Number(i64::MIN))
            .bind("MAX_INT".to_string(), LispVal::Number(i64::MAX));
        scope
    };
}


