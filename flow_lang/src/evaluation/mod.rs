use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::parsing::{error::LispValUnwrapError, LispVal};
use error::EvalError;

use self::scope::Scope;

pub mod scope;
pub mod error;

type EvalResult = Result<(Scope, LispVal), EvalError>;

trait EvalFn: Fn(Scope, &[LispVal]) -> EvalResult {}

impl<F> EvalFn for F where F: Fn(Scope, &[LispVal]) -> EvalResult {}

fn eval_op1<F: Fn(A1) -> R, A1, R>(operation: F) -> impl EvalFn
where
    A1: std::convert::TryFrom<LispVal, Error = LispValUnwrapError>,
    R: std::convert::Into<LispVal>,
{
    move |scope: Scope, values: &[LispVal]| {
        if values.len() != 1 {
            return Err(EvalError::InvalidArgumentsCount {
                expected: 1,
                got: values.len(),
            });
        }

        let a1 = values[0]
            .clone()
            .try_into()
            .map_err(|e| EvalError::from(e, 0))?; // values[0].as_number().unwrap(

        Ok((scope, operation(a1).into()))
    }
}

fn eval_op2<F: Fn(A1, A2) -> R, A1, A2, R>(operation: F) -> impl EvalFn
where
    A1: std::convert::TryFrom<LispVal, Error = LispValUnwrapError>,
    A2: std::convert::TryFrom<LispVal, Error = LispValUnwrapError>,
    R: std::convert::Into<LispVal>,
{
    move |scope: Scope, values: &[LispVal]| {
        if values.len() != 2 {
            return Err(EvalError::InvalidArgumentsCount {
                expected: 2,
                got: values.len(),
            });
        }

        let a1 = values[0]
            .clone()
            .try_into()
            .map_err(|e| EvalError::from(e, 0))?;
        let a2 = values[1]
            .clone()
            .try_into()
            .map_err(|e| EvalError::from(e, 1))?;

        Ok((scope, operation(a1, a2).into()))
    }
}

fn eval_fold(scope: Scope, values: &[LispVal]) -> EvalResult {
    let operation = &values[0];
    let initial = values[1].clone();
    let list: Vec<LispVal> = values[2].clone().try_into().unwrap();

    list.iter()
        .try_fold((scope, initial), |(scope, acc), value| {
            eval(
                scope,
                &LispVal::List(vec![operation.clone(), acc, value.clone()]),
            )
        })
}

fn concat_list(left: Vec<LispVal>, right: Vec<LispVal>) -> Vec<LispVal> {
    left.iter().chain(right.iter()).cloned().collect()
}

fn concat_string(left: String, right: String) -> String {
    format!("{}{}", left, right)
}

fn eval_concat(scope: Scope, values: &[LispVal]) -> EvalResult {
    let left = &values[0];
    let right = &values[1];

    match (left, right) {
        (LispVal::List(_), LispVal::List(_)) => eval_op2(concat_list)(scope, values),
        (LispVal::String(_), LispVal::String(_)) => eval_op2(concat_string)(scope, values),
        _ => Err(EvalError::InvalidConcatenation {
            left: left.to_type(),
            right: right.to_type(),
        }),
    }
}

fn eval_unevaluated(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 1 {
        return Err(EvalError::InvalidArgumentsCount {
            expected: 1,
            got: values.len(),
        });
    }

    eval(scope, &values[0])
}

fn eval_let(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 2 {
        return Err(EvalError::InvalidArgumentsCount {
            expected: 2,
            got: values.len(),
        });
    }

    let name: String = values[0].clone().try_into().map_err(|e| EvalError::from(e, 0))?;
    let value = values[1].clone();

    Ok((scope.bind(name, value.clone()), value))
}

fn eval_math<F>(operation: F) -> impl EvalFn
where
    F: Fn(i64, i64) -> i64,
{
    eval_op2(operation)
}

fn eval_logic<F>(operation: F) -> impl EvalFn
where
    F: Fn(bool, bool) -> bool,
{
    eval_op2(operation)
}

fn eval_comparison<F>(operation: F) -> impl EvalFn
where
    F: Fn(i64, i64) -> bool,
{
    eval_op2(operation)
}

lazy_static! {
    static ref SYMBOLS_TABLE: HashMap::<&'static str, Box<dyn EvalFn + Sync>> = {
        let mut s = HashMap::<&'static str, Box<dyn EvalFn + Sync>>::new();
        s.insert("eval", Box::new(eval_unevaluated));
        s.insert("print", Box::new(eval_op1(|s: String| print!("{}", s))));
        s.insert("to_string", Box::new(eval_op1(|n: i64| n.to_string())));
        s.insert("fold", Box::new(eval_fold));
        s.insert("concat", Box::new(eval_concat));
        s.insert("let", Box::new(eval_let));

        s.insert("+", Box::new(eval_math(|a, b| a + b)));
        s.insert("-", Box::new(eval_math(|a, b| a - b)));
        s.insert("*", Box::new(eval_math(|a, b| a * b)));
        s.insert("/", Box::new(eval_math(|a, b| a / b)));
        s.insert("%", Box::new(eval_math(|a, b| a % b)));

        s.insert("add", Box::new(eval_math(|a, b| a + b)));
        s.insert("sub", Box::new(eval_math(|a, b| a - b)));
        s.insert("mul", Box::new(eval_math(|a, b| a * b)));
        s.insert("div", Box::new(eval_math(|a, b| a / b)));
        s.insert("mod", Box::new(eval_math(|a, b| a % b)));
        s.insert("max", Box::new(eval_math(|a, b| a.max(b))));
        s.insert("min", Box::new(eval_math(|a, b| a.min(b))));

        s.insert("<", Box::new(eval_comparison(|a, b| a < b)));
        s.insert(">", Box::new(eval_comparison(|a, b| a > b)));
        s.insert("<=", Box::new(eval_comparison(|a, b| a <= b)));
        s.insert(">=", Box::new(eval_comparison(|a, b| a >= b)));
        s.insert("=", Box::new(eval_comparison(|a, b| a == b)));

        s.insert("lt", Box::new(eval_comparison(|a, b| a < b)));
        s.insert("gt", Box::new(eval_comparison(|a, b| a > b)));
        s.insert("ltq", Box::new(eval_comparison(|a, b| a <= b)));
        s.insert("gtq", Box::new(eval_comparison(|a, b| a >= b)));
        s.insert("eq", Box::new(eval_comparison(|a, b| a == b)));

        s.insert("and", Box::new(eval_logic(|a, b| a & b)));
        s.insert("or", Box::new(eval_logic(|a, b| a | b)));
        s.insert("not", Box::new(eval_op1(|a: bool| !a)));
        s
    };
}

fn eval_list(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.is_empty() {
        return Ok((scope, vec![].into()));
    }

    let (head, tail) = values.split_first().unwrap();

    if let LispVal::Symbol(atom) = head {
        let evaluated_list = tail
            .iter()
            .map(|v| eval(scope.clone(), v))
            .collect::<Result<Vec<_>, _>>()?;

        let (scopes, args): (Vec<_>, Vec<_>) = evaluated_list
            .into_iter()
            .unzip();

        let scope = scopes.into_iter().fold(Scope::new(), |acc, s| acc.merge(s));

        return match SYMBOLS_TABLE.get(atom.as_str()) {
            Some(f) => f(scope, &args),
            None => Err(EvalError::UnknownIdentifier(atom.clone())),
        };
    };

    Err(EvalError::InvalidFunctionCall {
        values: values.to_vec(),
    })
}

pub fn eval(scope: Scope, expr: &LispVal) -> EvalResult {
    match expr {
        LispVal::Symbol(atom) => match scope.get(atom.as_str()) {
            Some(value) => Ok((scope.clone(), value.clone())),
            None => Err(EvalError::UnknownIdentifier(atom.clone())),
        },
        LispVal::List(elements) => eval_list(scope, elements),
        LispVal::Unevaluated(value) => Ok((scope, *value.clone())),
        _ => Ok((scope, expr.clone())),
    }
}
