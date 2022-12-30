use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, one_of},
    combinator::{map, map_res, opt, recognize},
    error::context,
    multi::{many0, many0_count, many1},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};
use crate::parsing::string::parse_string;

use self::error::LispValUnwrapError;

mod string;
pub mod error;

#[derive(Debug, PartialEq, Clone)]
pub enum LispVal {
    Symbol(String),
    String(String),
    List(Vec<LispVal>),
    Number(i64),
    Boolean(bool),
    Unevaluated(Box<LispVal>),
    Void(),
}

#[derive(PartialEq, Debug)]
pub enum LispType {
    Any,
    Symbol,
    String,
    List,
    Number,
    Boolean,
    Void,
}

impl std::fmt::Display for LispType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispType::Any => write!(f, "any"),
            LispType::Symbol => write!(f, "symbol"),
            LispType::String => write!(f, "string"),
            LispType::List => write!(f, "list"),
            LispType::Number => write!(f, "number"),
            LispType::Boolean => write!(f, "boolean"),
            LispType::Void => write!(f, "void"),
        }
    }
}

impl std::fmt::Display for LispVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispVal::Void() => write!(f, ""),
            LispVal::Symbol(atom) => write!(f, "{}", atom),
            LispVal::Number(n) => write!(f, "{}", n.to_string()),
            LispVal::String(s) => write!(f, "{}", s.to_string()),
            LispVal::Unevaluated(expr) => write!(f, "'{}", expr.to_string()),
            LispVal::Boolean(b) => write!(f, "{}", b.to_string()),
            LispVal::List(values) => write!(
                f,
                "({})",
                values
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

impl LispVal {
    pub fn as_symbol(&self) -> Result<&str, LispValUnwrapError> {
        match self {
            Self::Symbol(s) => Ok(s),
            _ => Err(LispValUnwrapError { got: self.to_type(), expected: LispType::Symbol }),
        }
    }

    pub fn to_type(&self) -> LispType {
        match self {
            Self::Void() => LispType::Void,
            Self::Symbol(_) => LispType::Symbol,
            Self::Number(_) => LispType::Number,
            Self::String(_) => LispType::String,
            Self::List(_) => LispType::List,
            Self::Boolean(_) => LispType::Boolean,
            Self::Unevaluated(v) => v.to_type(),
        }
    }

    pub fn to_unevaluated(&self) -> Self {
        Self::Unevaluated(Box::new(self.clone()))
    }

    pub fn concat(&self, other: &Self) -> Self {
        match (self, other) {
            (LispVal::List(left), LispVal::List(right)) => {
                left.iter().chain(right.iter()).cloned().collect()
            }
            (LispVal::List(left), v) => {
                let mut result = left.clone();
                result.push(v.clone());
                result.into()
            }
            (v, LispVal::List(right)) => {
                let mut result = vec![v.clone()];
                result.extend(right.iter().cloned());
                result.into()
            }
            (l, r) => vec![l.clone(), r.clone()].into(),
        }
    }

    pub fn try_push(&self, other: &Self) -> Result<Self, LispValUnwrapError> {
        let mut list: Vec<LispVal> = self.clone().try_into()?;
        list.push(other.clone());
        Ok(list.into())
    }

    pub fn try_append(&self, other: &Vec<Self>) -> Result<Self, LispValUnwrapError> {
        let mut list: Vec<LispVal> = self.clone().try_into()?;
        list.extend(other.iter().cloned());
        Ok(list.into())
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void())
    }
}

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

fn parse_symbol(input: &str) -> IResult<&str, &str> {
    let parse_operators = recognize(many1(one_of("><+-*/%=")));
    let parse_identifier = recognize(pair(
        alt((alpha1, tag("_"))),
        terminated(many0_count(alt((alphanumeric1, tag("_")))), opt(one_of("?!"))),
    ));

    context("symbol", alt((parse_operators, parse_identifier)))(input)
}

fn parse_boolean(input: &str) -> IResult<&str, bool> {
    context(
        "boolean",
        alt((map(tag("true"), |_| true), map(tag("false"), |_| false))),
    )(input)
}

fn parse_number(input: &str) -> IResult<&str, i64> {
    context(
        "number",
        map_res(
            recognize(preceded(opt(alt((char('-'), char('+')))), digit1)),
            str::parse::<i64>,
        ),
    )(input)
}

fn parse_list<'a>(input: &str) -> IResult<&str, Vec<LispVal>> {
    context(
        "list",
        delimited(char('('), many0(parse_expression), char(')')),
    )(input)
}

fn parse_unevaluated(input: &str) -> IResult<&str, LispVal> {
    context(
        "unevaluated",
        preceded(
            char('\''),
            map(parse_expression, |v| LispVal::Unevaluated(Box::new(v))),
        ),
    )(input)
}

fn parse_expression<'a>(input: &str) -> IResult<&str, LispVal> {
    context(
        "expression",
        delimited(
            opt(multispace0),
            alt((
                parse_unevaluated,
                map(parse_boolean, LispVal::Boolean),
                map(parse_number, LispVal::Number),
                map(parse_symbol, |v| LispVal::Symbol(v.into())),
                map(parse_string, |v| LispVal::String(v.into())),
                map(parse_list, |v| LispVal::List(v.into())),
            )),
            opt(multispace0),
        ),
    )(input)
}

pub fn parse(input: &str) -> IResult<&str, LispVal> {
    terminated(parse_expression, multispace0)(input)
}

#[macro_export]
macro_rules! parse_it {
    ($input:expr) => {
        crate::parsing::parse($input).map(|(_, v)| v).unwrap()
    };
}

#[cfg(test)]
mod tests {
    use crate::parsing::LispVal;

    #[test]
    fn test_math_expression() {
        assert_eq!(parse_it!("(+ 1 2)"), LispVal::List(vec![
            LispVal::Symbol("+".into()),
            LispVal::Number(1),
            LispVal::Number(2),
        ]));
    }

    #[test]
    fn test_nested_math_expression() {
        assert_eq!(parse_it!("(+ 1 (* 2 3))"), LispVal::List(vec![
            LispVal::Symbol("+".into()),
            LispVal::Number(1),
            LispVal::List(vec![
                LispVal::Symbol("*".into()),
                LispVal::Number(2),
                LispVal::Number(3),
            ]),
        ]));
    }

    #[test]
    fn test_unevaluated_expression() {
        assert_eq!(parse_it!("'(+ 1 2)"), LispVal::Unevaluated(Box::new(LispVal::List(vec![
            LispVal::Symbol("+".into()),
            LispVal::Number(1),
            LispVal::Number(2),
        ]))));
    }

    #[test]
    fn test_boolean() {
        assert_eq!(parse_it!("true"), LispVal::Boolean(true));
        assert_eq!(parse_it!("false"), LispVal::Boolean(false));
    }

    #[test]
    fn test_number() {
        assert_eq!(parse_it!("1"), LispVal::Number(1));
        assert_eq!(parse_it!("+1"), LispVal::Number(1));
        assert_eq!(parse_it!("-1"), LispVal::Number(-1));
    }
}