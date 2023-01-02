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
use crate::{parsing::string::parse_string};

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
    Function { parameters: Vec<String>, body: Box<LispVal>, applied: Vec<LispVal> },
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
    Function,
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
            LispType::Function => write!(f, "function"),
            LispType::Void => write!(f, "void"),
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
            Self::Function { .. } => LispType::Function,
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

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void())
    }

    pub fn is_macro(&self) -> bool {
        matches!(self, Self::Symbol(v) if v.ends_with("!"))
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