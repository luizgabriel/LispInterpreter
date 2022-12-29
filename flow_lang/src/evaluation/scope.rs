use lazy_static::lazy_static;

use crate::parsing::LispVal;

#[derive(Clone, Debug)]
pub struct Scope {
    bindings: im::HashMap<String, LispVal>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            bindings: im::HashMap::<String, LispVal>::new(),
        }
    }

    pub fn bind(&self, name: String, value: LispVal) -> Scope {
        Scope {
            bindings: self.bindings.update(name, value),
        }
    }

    pub fn merge(&self, other: Scope) -> Scope {
        Scope {
            bindings: self.bindings.clone().union(other.bindings),
        }
    }

    pub fn get(&self, name: &str) -> Option<&LispVal> {
        self.bindings.get(name)
    }
}

lazy_static! {
    pub static ref INITIAL_SCOPE: Scope = Scope::new()
        .bind("MIN_INT".to_string(), LispVal::Number(i64::MIN))
        .bind("MAX_INT".to_string(), LispVal::Number(i64::MAX));
}


