use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::parsing::{error::LispValUnwrapError, LispVal};
use error::EvalError;

use self::scope::{Scope, INITIAL_SCOPE};

pub mod error;
pub mod scope;

type EvalResult = Result<(Scope, LispVal), EvalError>;

trait EvalFn: Fn(Scope, &[LispVal]) -> EvalResult {}

impl<F> EvalFn for F where F: Fn(Scope, &[LispVal]) -> EvalResult {}

fn eval_op1<F: Fn(A1) -> R, A1, R>(operation: F) -> impl EvalFn
where
    A1: std::convert::TryFrom<LispVal, Error = LispValUnwrapError>,
    R: std::convert::Into<LispVal>,
{
    move |scope: Scope, values: &[LispVal]| -> EvalResult {
        let name = scope.context.clone();

        if values.len() != 1 {
            return Err(EvalError::InvalidArgumentsCount {
                name,
                expected: 1,
                got: values.len(),
            });
        }

        let a1 = values[0].clone().try_into().map_err(EvalError::from_arg(0, &name))?;

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
        let name = scope.context.clone();

        if values.len() != 2 {
            return Err(EvalError::InvalidArgumentsCount {
                name,
                expected: 2,
                got: values.len(),
            });
        }

        let a1 = values[0].clone().try_into().map_err(EvalError::from_arg(0, &name))?;
        let a2 = values[1].clone().try_into().map_err(EvalError::from_arg(1, &name))?;

        Ok((scope, operation(a1, a2).into()))
    }
}

fn eval_fold(scope: Scope, values: &[LispVal]) -> EvalResult {
    let name = scope.context.clone();

    if values.len() != 3 {
        return Err(EvalError::InvalidArgumentsCount {
            name,
            expected: 3,
            got: values.len(),
        });
    }

    let operation: Vec<LispVal> = values[0].clone().try_into().map_err(EvalError::from_arg(0, &name))?;
    let initial = values[1].clone();
    let list: Vec<LispVal> = values[2].clone().try_into().map_err(EvalError::from_arg(1, &name))?;

    list.iter()
        .try_fold((scope, initial), |(scope, acc), value| {
            let mut expr = operation.clone();
            expr.push(acc.clone());
            expr.push(value.clone());
            eval(scope, &expr.into()) 
        })
}

fn eval_map(scope: Scope, values: &[LispVal]) -> EvalResult {
    let name = scope.context.clone();

    if values.len() != 2 {
        return Err(EvalError::InvalidArgumentsCount {
            name,
            expected: 2,
            got: values.len(),
        });
    }

    let operation: Vec<LispVal> = values[0].clone().try_into().map_err(EvalError::from_arg(0, &name))?;
    let list: Vec<LispVal> = values[1].clone().try_into().map_err(EvalError::from_arg(1, &name))?;

    let (scope, list) = list
        .into_iter()
        .try_fold((scope, vec![]), |(scope, mut acc), value| {
            let mut expr = operation.clone();
            expr.push(value);

            let (scope, result) = eval(scope, &expr.into())?;
            acc.push(result);

            Ok((scope, acc))
        })?;

    return Ok((scope, list.into()));
}

fn eval_if(scope: Scope, values: &[LispVal]) -> EvalResult {
    let name = scope.context.clone();

    if values.len() != 3 {
        return Err(EvalError::InvalidArgumentsCount {
            name,
            expected: 3,
            got: values.len(),
        });
    }
    let (scope, condition) = eval(scope, &values[0])?;
    let condition = condition.try_into().map_err(EvalError::from_arg(0, &name))?;

    if condition {
        eval(scope, &values[1])
    } else {
        eval(scope, &values[2])
    }
}

fn eval_concat(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 2 {
        return Err(EvalError::InvalidArgumentsCount {
            name: scope.context,
            expected: 2,
            got: values.len(),
        });
    }

    let (scope, left) = eval(scope, &values[0])?;
    let (scope, right) = eval(scope, &values[1])?;

    Ok((scope, left.concat(&right)))
}

fn eval_unevaluated(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 1 {
        return Err(EvalError::InvalidArgumentsCount {
            name: scope.context,
            expected: 1,
            got: values.len(),
        });
    }

    eval(scope, &values[0])
}

fn eval_let(scope: Scope, values: &[LispVal]) -> EvalResult {
    let name = scope.context.clone();

    if values.len() != 2 {
        return Err(EvalError::InvalidArgumentsCount {
            name,
            expected: 2,
            got: values.len(),
        });
    }

    let name = values[0].as_symbol().map_err(EvalError::from_arg(0, &name))?;
    let value = values[1].clone();

    Ok((scope.bind(name.to_string(), value), LispVal::Void()))
}

fn eval_print_scope(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 0 {
        return Err(EvalError::InvalidArgumentsCount {
            name: scope.context,
            expected: 0,
            got: values.len(),
        });
    }

    println!("{:#?}", scope);

    Ok((scope, LispVal::Void()))
}

fn eval_clear_scope(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 0 {
        return Err(EvalError::InvalidArgumentsCount {
            name: scope.context,
            expected: 0,
            got: values.len(),
        });
    }

    Ok((INITIAL_SCOPE.clone(), LispVal::Void()))
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

fn eval_push(scope: Scope, values: &[LispVal]) -> Result<(Scope, LispVal), EvalError> {
    let name = scope.context.clone();

    if values.len() != 2 {
        return Err(EvalError::InvalidArgumentsCount {
            name,
            expected: 2,
            got: values.len(),
        });
    }

    let mut list: Vec<LispVal> = values[0].clone().try_into().map_err(EvalError::from_arg(0, &name))?;
    let value = values[1].clone();

    list.push(value);

    Ok((scope, list.into()))
}

lazy_static! {
    static ref SYMBOLS_TABLE: HashMap::<&'static str, Box<dyn EvalFn + Sync>> = {
        let mut s = HashMap::<&'static str, Box<dyn EvalFn + Sync>>::new();
        s.insert("eval", Box::new(eval_unevaluated));
        s.insert(
            "print",
            Box::new(eval_op1(|s: String| println!("{}", s))),
        );
        s.insert(
            "to_string",
            Box::new(eval_op1(|n: i64| n.to_string())),
        );
        s.insert("fold", Box::new(eval_fold));
        s.insert("map", Box::new(eval_map));
        s.insert("concat", Box::new(eval_concat));
        s.insert("push", Box::new(eval_push));
        s.insert("let", Box::new(eval_let));
        s.insert("_scope", Box::new(eval_print_scope));
        s.insert("_clear", Box::new(eval_clear_scope));
        s.insert(
            "head",
            Box::new(eval_op1(|l: Vec<LispVal>| {
                l.get(0).unwrap().clone()
            })),
        );
        s.insert(
            "tail",
            Box::new(eval_op1(|l: Vec<LispVal>| l[1..].to_vec())),
        );
        s.insert(
            "len",
            Box::new(eval_op1(|l: Vec<LispVal>| l.len() as i64)),
        );
        s.insert("if", Box::new(eval_if));

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

    let (heads, tail) = values.clone().split_at(1);
    let head = heads.get(0).unwrap();

    if let LispVal::Symbol(atom) = head {
        let (scope, tail) = eval_tail(scope, tail)?;

        if atom == "list" {
            return Ok((scope, tail.into()));
        }

        if let Some(f) = SYMBOLS_TABLE.get(atom.as_str()) {
            return f(scope.with_context(atom.clone()), &tail);
        };

        if let Some(value) = scope.get(atom.as_str()) {
            let expr = value.try_append(&tail).map_err(EvalError::from_invoke(&tail, atom))?;
            return eval(scope.with_context(atom.to_string()), &expr);
        };

        return Err(EvalError::UnknownIdentifier(atom.clone()))
    };

    Err(EvalError::InvalidFunctionCall {
        values: values.to_vec(),
    })
}

fn eval_tail(scope: Scope, tail: &[LispVal]) -> Result<(Scope, Vec<LispVal>), EvalError> {
    tail.into_iter()
        .try_fold((scope, Vec::<LispVal>::new()), |(scope, acc), value| {
            let (scope, value) = eval(scope, &value)?;
            Ok((scope, {
                let mut acc = acc;
                acc.push(value);
                acc
            }))
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

#[macro_export]
macro_rules! eval_it {
    ($expr:expr) => {
        crate::evaluation::eval($crate::evaluation::scope::Scope::new("unknown".to_string()), &parse_it!($expr))
            .unwrap()
            .1
    };
    ($expr:expr, $scope:expr) => {
        crate::evaluation::eval($scope, &parse_it!($expr))
            .unwrap()
            .1
    };
}

#[cfg(test)]
mod tests {
    use crate::{parse_it, parsing::LispVal};

    #[test]
    fn test_math_expression() {
        assert_eq!(eval_it!("(+ 1 2)"), LispVal::Number(3));
    }

    #[test]
    fn test_binding() {
        assert_eq!(eval_it!("(list (let 'x 10) (+ x 2))"), vec![
            LispVal::Void(),
            LispVal::Number(12)
        ].into());
    }

    #[test]
    fn test_fold() {
        assert_eq!(eval_it!("(fold '(+) 1 '(1 2 3))"), LispVal::Number(7));
    }

    #[test]
    fn test_map() {
        assert_eq!(eval_it!("(map '(+ 2) '(1 2 3))"), vec![
            LispVal::Number(3),
            LispVal::Number(4),
            LispVal::Number(5)
        ].into());
    }
}
