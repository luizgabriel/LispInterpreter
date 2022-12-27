mod string;

use crate::parsing::string::parse_string;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, many0_count},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum LispVal {
    Symbol(String),
    String(String),
    List(Vec<LispVal>),
    Number(i64),
    Unevaluated(Box<LispVal>),
    Void(),
}

impl std::fmt::Display for LispVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispVal::Void() => write!(f, ""),
            LispVal::Symbol(atom) => write!(f, "{}", atom),
            LispVal::Number(n) => write!(f, "{}", n.to_string()),
            LispVal::String(s) => write!(f, "{}", s.to_string()),
            LispVal::Unevaluated(expr) => expr.fmt(f),
            LispVal::List(values) => write!(
                f,
                "({})",
                values.iter().fold(String::new(), |acc, cur| format!(
                    "{} {}",
                    acc,
                    cur.to_string()
                ))
            ),
        }
    }
}

impl LispVal {
    pub fn as_number(&self) -> Option<i64> {
        if let Self::Number(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<String> {
        if let Self::String(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<Vec<LispVal>> {
        if let Self::List(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn to_type_name(&self) -> &'static str {
        match self {
            Self::Void() => "void",
            Self::Symbol(_) => "atom",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::List(_) => "list",
            Self::Unevaluated(v) => v.to_type_name(),
        }
    }
}

fn parse_atom(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parse_number(input: &str) -> IResult<&str, i64> {
    alt((
        map(tag("true"), |_| 1i64),
        map(tag("false"), |_| 0i64),
        map_res(recognize(digit1), str::parse::<i64>),
    ))(input)
}

fn parse_list<'a>(input: &str) -> IResult<&str, Vec<LispVal>> {
    delimited(char('('), many0(parse_expression), char(')'))(input)
}

fn parse_unevaluated(input: &str) -> IResult<&str, LispVal> {
    preceded(
        char('\''),
        map(parse_expression, |v| LispVal::Unevaluated(Box::new(v))),
    )(input)
}

fn parse_expression<'a>(input: &str) -> IResult<&str, LispVal> {
    preceded(
        opt(multispace0),
        alt((
            parse_unevaluated,
            map(parse_number, LispVal::Number),
            map(parse_atom, |v| LispVal::Symbol(v.into())),
            map(parse_string, |v| LispVal::String(v.into())),
            map(parse_list, LispVal::List),
        )),
    )(input)
}

pub fn parse(input: &str) -> IResult<&str, LispVal> {
    terminated(parse_expression, multispace0)(input)
}
