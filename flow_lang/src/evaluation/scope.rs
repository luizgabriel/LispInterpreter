use lazy_static::lazy_static;

use crate::parsing::LispVal;

#[derive(Clone, Debug)]
pub struct Scope {
    pub context: String,
    bindings: im::HashMap<String, LispVal>,
}

impl Scope {
    pub fn new(context: String) -> Scope {
        Scope {
            context,
            bindings: im::HashMap::<String, LispVal>::new(),
        }
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

    pub fn merge(&self, other: Scope) -> Scope {
        if other.context != self.context {
            panic!("Cannot merge scopes with different contexts");
        }

        Scope {
            context: self.context.clone(),
            bindings: self.bindings.clone().union(other.bindings),
        }
    }

    pub fn get(&self, name: &str) -> Option<&LispVal> {
        self.bindings.get(name)
    }
}

lazy_static! {
    pub static ref INITIAL_SCOPE: Scope = Scope::new("main".to_string())
        .bind("MIN_INT".to_string(), LispVal::Number(i64::MIN))
        .bind("MAX_INT".to_string(), LispVal::Number(i64::MAX));
}


