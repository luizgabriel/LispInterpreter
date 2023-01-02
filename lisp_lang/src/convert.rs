use crate::{parsing::{LispVal, error::LispValUnwrapError, LispType}};

impl FromIterator<LispVal> for LispVal {
    fn from_iter<T: IntoIterator<Item = LispVal>>(iter: T) -> Self {
        Self::List(iter.into_iter().collect())
    }
}

impl From<i64> for LispVal {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for LispVal {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<String> for LispVal {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<Vec<LispVal>> for LispVal {
    fn from(v: Vec<LispVal>) -> Self {
        Self::List(v)
    }
}

impl From<()> for LispVal {
    fn from(_: ()) -> Self {
        Self::Void()
    }
}

impl TryFrom<LispVal> for i64 {
    type Error = LispValUnwrapError;

    fn try_from(value: LispVal) -> Result<Self, Self::Error> {
        match value {
            LispVal::Number(n) => Ok(n),
            _ => Err(LispValUnwrapError {
                expected: LispType::Number,
                got: value.to_type(),
            }),
        }
    }
}

impl TryFrom<LispVal> for bool {
    type Error = LispValUnwrapError;

    fn try_from(value: LispVal) -> Result<Self, Self::Error> {
        match value {
            LispVal::Boolean(b) => Ok(b),
            _ => Err(LispValUnwrapError {
                expected: LispType::Boolean,
                got: value.to_type(),
            }),
        }
    }
}

impl TryFrom<LispVal> for String {
    type Error = LispValUnwrapError;

    fn try_from(value: LispVal) -> Result<Self, Self::Error> {
        match value {
            LispVal::String(s) => Ok(s),
            _ => Err(LispValUnwrapError {
                expected: LispType::String,
                got: value.to_type(),
            }),
        }
    }
}

impl TryFrom<LispVal> for Vec<LispVal> {
    type Error = LispValUnwrapError;

    fn try_from(value: LispVal) -> Result<Self, Self::Error> {
        match value {
            LispVal::List(v) => Ok(v),
            _ => Err(LispValUnwrapError {
                expected: LispType::List,
                got: value.to_type(),
            }),
        }
    }
}

impl TryFrom<LispVal> for () {
    type Error = LispValUnwrapError;

    fn try_from(value: LispVal) -> Result<Self, Self::Error> {
        match value {
            LispVal::Void() => Ok(()),
            _ => Err(LispValUnwrapError {
                expected: LispType::Void,
                got: value.to_type(),
            }),
        }
    }
}