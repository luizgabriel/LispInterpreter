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

impl LispVal {
    fn into_value<T>(self, position: usize, scope: &Scope) -> Result<T, EvalError>
    where
        T: TryFrom<LispVal, Error = LispValUnwrapError>,
    {
        self.try_into().map_err(|e| EvalError::from(e, position, scope))
    }
}

fn eval_op1<F: Fn(A1) -> R, A1, R>(operation: F) -> impl EvalFn
where
    A1: std::convert::TryFrom<LispVal, Error = LispValUnwrapError>,
    R: std::convert::Into<LispVal>,
{
    move |scope: Scope, values: &[LispVal]| -> EvalResult {
        if values.len() != 1 {
            return Err(EvalError::InvalidArgumentsCount {
                name: scope.context,
                expected: 1,
                got: values.len(),
            });
        }

        let (scope, arg1) = eval(scope, &values[0])?;
        let a1 = arg1.into_value(0, &scope)?;

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
                name: scope.context,
                expected: 2,
                got: values.len(),
            });
        }

        let (scope, arg1) = eval(scope, &values[0])?;
        let (scope, arg2) = eval(scope, &values[1])?;
        let a1 = arg1.into_value(0, &scope)?;
        let a2 = arg2.into_value(1, &scope)?;

        Ok((scope, operation(a1, a2).into()))
    }
}

fn eval_fold(scope: Scope, values: &[LispVal]) -> EvalResult {
    let (scope, operation) = eval(scope, &values[0])?;
    let (scope, initial) = eval(scope, &values[1])?;
    let (scope, list) = eval(scope, &values[2])?;
    let list: Vec<LispVal> = list.into_value(2, &scope)?;

    list.iter()
        .try_fold((scope, initial), |(scope, acc), value| {
            let expr = vec![operation.clone(), acc.clone(), value.clone()].into();
            eval(scope, &expr) // (eval '(op acc value))
        })
}

fn eval_map(scope: Scope, values: &[LispVal]) -> EvalResult {
    let operation = &values[0];
    let (scope, arg2) = eval(scope, &values[1])?;

    let list: Vec<LispVal> = arg2.try_into().unwrap();
    let initial = (scope, vec![]);

    let (scope, list) = list
        .into_iter()
        .try_fold(initial, |(scope, mut acc), value| {
            let (scope, expr) =
                eval_concat(scope, [operation.clone(), value.to_unevaluated()].as_ref())?;
            let (scope, result) = eval(scope, &expr)?;
            acc.push(result);

            Ok((scope, acc))
        })?;

    return Ok((scope, list.into()));
}

fn eval_if(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 3 {
        return Err(EvalError::InvalidArgumentsCount {
            name: "if".to_string(),
            expected: 3,
            got: values.len(),
        });
    }
    let (scope, condition) = eval(scope, &values[0])?;
    let condition = condition.into_value(0, &scope)?;

    if condition {
        eval(scope, &values[1])
    } else {
        eval(scope, &values[2])
    }
}

fn eval_concat(scope: Scope, values: &[LispVal]) -> EvalResult {
    let (scope, mut left) = eval(scope, &values[0])?;
    let (scope, mut right) = eval(scope, &values[1])?;

    match (&mut left, &mut right) {
        (LispVal::List(left), LispVal::List(right)) => {
            left.append(right);
            Ok((scope, left.clone().into()))
        }
        (LispVal::List(left), v) => {
            left.push(v.clone());
            Ok((scope, left.clone().into()))
        }
        (v, LispVal::List(right)) => {
            let mut list = vec![v.clone()];
            list.append(right);
            Ok((scope, list.into()))
        }
        (l, r) => Ok((scope, vec![l.clone(), r.clone()].into())),
    }
}

fn eval_unevaluated(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 1 {
        return Err(EvalError::InvalidArgumentsCount {
            name: "eval".to_string(),
            expected: 1,
            got: values.len(),
        });
    }

    eval(scope, &values[0])
}

fn eval_let(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.len() != 2 {
        return Err(EvalError::InvalidArgumentsCount {
            name: "let".to_string(),
            expected: 2,
            got: values.len(),
        });
    }

    let name = values[0]
        .as_symbol()
        .map_err(|e| EvalError::from(e, 0, &scope))?;
    let (scope, value) = eval(scope, &values[1])?;

    Ok((scope.bind(name.to_string(), value.clone()), value))
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

lazy_static! {
    static ref SYMBOLS_TABLE: HashMap::<&'static str, Box<dyn EvalFn + Sync>> = {
        let mut s = HashMap::<&'static str, Box<dyn EvalFn + Sync>>::new();
        s.insert("eval", Box::new(eval_unevaluated));
        s.insert(
            "print",
            Box::new(eval_op1(|s: String| print!("{}", s))),
        );
        s.insert(
            "to_string",
            Box::new(eval_op1(|n: i64| n.to_string())),
        );
        s.insert("fold", Box::new(eval_fold));
        s.insert("map", Box::new(eval_map));
        s.insert("concat", Box::new(eval_concat));
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
        if atom == "list" {
            let (scope, tail) = tail.into_iter()
                .try_fold((scope, Vec::<LispVal>::new()), |(scope, acc), value| {
                    let (scope, value) = eval(scope, &value)?;
                    Ok((scope, {
                        let mut acc = acc;
                        acc.push(value);
                        acc
                    }))
                })?;

            return Ok((scope, tail.into()));
        }

        return match SYMBOLS_TABLE.get(atom.as_str()) {
            Some(f) => f(scope.with_context(atom.clone()), &tail),
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
        assert_eq!(eval_it!("(+ 1 2 3 4 5)"), LispVal::Number(15));
    }

    #[test]
    fn test_binding() {
        assert_eq!(eval_it!("(list (let x 10) (+ x 2))"), LispVal::List(vec![
            LispVal::Number(10),
            LispVal::Number(12)
        ]));
    }
}
