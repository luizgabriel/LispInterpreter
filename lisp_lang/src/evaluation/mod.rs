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
        let a1 = values
            .get(0)
            .unwrap()
            .clone()
            .try_into()
            .map_err(EvalError::from_arg(0, &name))?;

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
        let a1 = values
            .get(0)
            .unwrap()
            .clone()
            .try_into()
            .map_err(EvalError::from_arg(0, &name))?;
        let a2 = values
            .get(1)
            .unwrap()
            .clone()
            .try_into()
            .map_err(EvalError::from_arg(1, &name))?;

        Ok((scope, operation(a1, a2).into()))
    }
}

fn eval_fold(scope: Scope, values: &[LispVal]) -> EvalResult {
    let name = scope.context.clone();
    let operation = values.get(0).unwrap();
    let initial = values.get(1).unwrap().clone();
    let list: Vec<LispVal> = values
        .get(2)
        .unwrap()
        .clone()
        .try_into()
        .map_err(EvalError::from_arg(1, &name))?;

    list.iter()
        .try_fold((scope, initial), |(scope, acc), value| {
            eval(scope, &vec![operation.clone(), acc, value.clone()].into())
        })
}

fn eval_map(scope: Scope, values: &[LispVal]) -> EvalResult {
    let name = scope.context.clone();

    let operation = values.get(0).unwrap().clone();

    let list: Vec<LispVal> = values
        .get(1)
        .unwrap()
        .clone()
        .try_into()
        .map_err(EvalError::from_arg(1, &name))?;

    let (scope, list) = list
        .into_iter()
        .try_fold((scope, Vec::new()), |(scope, mut acc), value| {
            let (scope, result) = eval(scope, &vec![operation.clone(), value.clone()].into())?;
            acc.push(result);
            Ok((scope, acc))
        })?;

    return Ok((scope, list.into()));
}

fn eval_if(scope: Scope, values: &[LispVal]) -> EvalResult {
    let name = scope.context.clone();
    let (scope, condition) = eval(scope, &values.get(0).unwrap())?;
    let condition = condition
        .try_into()
        .map_err(EvalError::from_arg(0, &name))?;

    if condition {
        eval(scope, &values.get(1).unwrap())
    } else {
        eval(scope, &values.get(2).unwrap())
    }
}

fn eval_concat(scope: Scope, values: &[LispVal]) -> EvalResult {
    let (scope, left) = eval(scope, &values.get(0).unwrap())?;
    let (scope, right) = eval(scope, &values.get(1).unwrap())?;

    Ok((scope, left.concat(&right)))
}

fn eval_unevaluated(scope: Scope, values: &[LispVal]) -> EvalResult {
    eval(scope, &values.get(0).unwrap())
}

fn eval_value_definition(scope: Scope, values: &[LispVal]) -> EvalResult {
    let name = scope.context.clone();
    let name = values
        .get(0)
        .unwrap()
        .as_symbol()
        .map_err(EvalError::from_arg(0, &name))?;
    let value = values.get(1).unwrap().clone();

    Ok((scope.bind(name.to_string(), value), LispVal::Void()))
}

fn eval_print_scope(scope: Scope, _: &[LispVal]) -> EvalResult {
    println!("{}", scope);

    Ok((scope, LispVal::Void()))
}

fn eval_clear_scope(_: Scope, _: &[LispVal]) -> EvalResult {
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
    let mut list: Vec<LispVal> = values
        .get(0)
        .unwrap()
        .clone()
        .try_into()
        .map_err(EvalError::from_arg(0, &name))?;
    let value = values.get(1).unwrap().clone();

    list.push(value);

    Ok((scope, list.into()))
}

fn eval_function_value(scope: Scope, values: &[LispVal]) -> Result<(Scope, LispVal), EvalError> {
    let name = scope.context.clone();
    let args_values: Vec<LispVal> = values
        .get(0)
        .unwrap()
        .clone()
        .try_into()
        .map_err(EvalError::from_arg(0, &name))?;
    let body = Box::new(values.get(1).unwrap().clone());

    let args = args_values
        .iter()
        .map(|v| {
            v.as_symbol()
                .map(|v| v.to_string())
                .map_err(EvalError::from_arg(1, &name))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((
        scope,
        LispVal::Function {
            parameters: args,
            body,
            applied: Vec::new(),
        },
    ))
}

fn eval_debug(scope: Scope, values: &[LispVal]) -> Result<(Scope, LispVal), EvalError> {
    let value = values.get(0).unwrap().clone();
    println!("{:#?}", value);
    Ok((scope, value))
}

fn eval_function_definition(
    scope: Scope,
    values: &[LispVal],
) -> Result<(Scope, LispVal), EvalError> {
    let name = scope.context.clone();
    let function_name = values
        .get(0)
        .unwrap()
        .as_symbol()
        .map_err(EvalError::from_arg(0, &name))?;
    let (scope, function) = eval_function_value(scope, &values[1..])?;

    Ok((
        scope.bind(function_name.to_string(), function),
        LispVal::Void(),
    ))
}

pub struct NativeFunction {
    pub required_arguments_count: usize,
    implementation: Box<dyn EvalFn + Sync>,
}

impl NativeFunction {
    fn new<F>(required_arguments_count: usize, function: F) -> Self
    where
        F: EvalFn + Sync + 'static,
    {
        Self {
            required_arguments_count,
            implementation: Box::new(function),
        }
    }

    fn to_function(&self, name: String, applied: Vec<LispVal>) -> LispVal {
        let args: Vec<_> = (0..self.required_arguments_count)
            .into_iter()
            .map(|n| format!("a{n}"))
            .collect();

        let mut expr = vec![name];
        expr.extend(args.clone());

        LispVal::Function {
            parameters: args,
            body: Box::new(LispVal::List(
                expr.iter()
                    .map(|v| LispVal::Symbol(v.to_string()))
                    .collect(),
            )),
            applied,
        }
    }

    fn call(&self, scope: Scope, values: &[LispVal]) -> EvalResult {
        let name = scope.context.clone();

        if values.len() < self.required_arguments_count {
            return Ok((scope, self.to_function(name.to_string(), values.to_vec())));
        }

        (self.implementation)(scope, &values)
    }
}

lazy_static! {
    static ref INTERNAL_SYMBOLS_TABLE: HashMap::<&'static str, NativeFunction> = {
        let mut s = HashMap::<&'static str, NativeFunction>::new();
        s.insert("eval", NativeFunction::new(1, eval_unevaluated));
        s.insert(
            "print",
            NativeFunction::new(1, eval_op1(|s: String| println!("{}", s))),
        );
        s.insert("debug", NativeFunction::new(1, eval_debug));
        s.insert(
            "to_string",
            NativeFunction::new(1, eval_op1(|n: i64| n.to_string())),
        );
        s.insert("fold", NativeFunction::new(3, eval_fold));
        s.insert("map", NativeFunction::new(2, eval_map));
        s.insert("concat", NativeFunction::new(2, eval_concat));
        s.insert("push", NativeFunction::new(2, eval_push));
        s.insert("fn!", NativeFunction::new(2, eval_function_value));
        s.insert("def!", NativeFunction::new(2, eval_value_definition));
        s.insert("defn!", NativeFunction::new(3, eval_function_definition));
        s.insert("print_scope", NativeFunction::new(0, eval_print_scope));
        s.insert("clear_scope", NativeFunction::new(0, eval_clear_scope));
        s.insert(
            "head",
            NativeFunction::new(1, eval_op1(|l: Vec<LispVal>| l.get(0).unwrap().clone())),
        );
        s.insert(
            "tail",
            NativeFunction::new(1, eval_op1(|l: Vec<LispVal>| l[1..].to_vec())),
        );
        s.insert(
            "len",
            NativeFunction::new(1, eval_op1(|l: Vec<LispVal>| l.len() as i64)),
        );
        s.insert("if!", NativeFunction::new(3, eval_if));

        s.insert("+", NativeFunction::new(2, eval_math(|a, b| a + b)));
        s.insert("-", NativeFunction::new(2, eval_math(|a, b| a - b)));
        s.insert("*", NativeFunction::new(2, eval_math(|a, b| a * b)));
        s.insert("/", NativeFunction::new(2, eval_math(|a, b| a / b)));
        s.insert("%", NativeFunction::new(2, eval_math(|a, b| a % b)));

        s.insert("add", NativeFunction::new(2, eval_math(|a, b| a + b)));
        s.insert("sub", NativeFunction::new(2, eval_math(|a, b| a - b)));
        s.insert("mul", NativeFunction::new(2, eval_math(|a, b| a * b)));
        s.insert("div", NativeFunction::new(2, eval_math(|a, b| a / b)));
        s.insert("mod", NativeFunction::new(2, eval_math(|a, b| a % b)));
        s.insert("max", NativeFunction::new(2, eval_math(|a, b| a.max(b))));
        s.insert("min", NativeFunction::new(2, eval_math(|a, b| a.min(b))));

        s.insert("<", NativeFunction::new(2, eval_comparison(|a, b| a < b)));
        s.insert(">", NativeFunction::new(2, eval_comparison(|a, b| a > b)));
        s.insert("<=", NativeFunction::new(2, eval_comparison(|a, b| a <= b)));
        s.insert(">=", NativeFunction::new(2, eval_comparison(|a, b| a >= b)));
        s.insert("=", NativeFunction::new(2, eval_comparison(|a, b| a == b)));

        s.insert("lt", NativeFunction::new(2, eval_comparison(|a, b| a < b)));
        s.insert("gt", NativeFunction::new(2, eval_comparison(|a, b| a > b)));
        s.insert(
            "ltq",
            NativeFunction::new(2, eval_comparison(|a, b| a <= b)),
        );
        s.insert(
            "gtq",
            NativeFunction::new(2, eval_comparison(|a, b| a >= b)),
        );
        s.insert("eq", NativeFunction::new(2, eval_comparison(|a, b| a == b)));

        s.insert("and", NativeFunction::new(2, eval_logic(|a, b| a & b)));
        s.insert("or", NativeFunction::new(2, eval_logic(|a, b| a | b)));
        s.insert("not", NativeFunction::new(1, eval_op1(|a: bool| !a)));
        s
    };
}

fn eval_function(
    scope: Scope,
    parameters: &[String],
    body: &LispVal,
    arguments: Vec<LispVal>,
) -> EvalResult {
    // Partial Function Application
    if arguments.len() < parameters.len() {
        return Ok((
            scope.clone(),
            LispVal::Function {
                parameters: parameters.to_vec(),
                body: Box::new(body.clone()),
                applied: arguments,
            },
        ));
    }

    let scope_before = scope.clone();

    // Bind arguments to scope
    let scope = parameters
        .iter()
        .zip(arguments)
        .fold(scope_before.clone(), |scope, (arg, value)| {
            scope.bind(arg.clone(), value.clone())
        });

    // Ignore the scope returned by the function
    let (_, result) = eval(scope, body)?;

    return Ok((scope_before, result));
}

fn eval_list(scope: Scope, values: &[LispVal]) -> EvalResult {
    if values.is_empty() {
        return Ok((scope, vec![].into()));
    }

    let (heads, tail) = values.clone().split_at(1);
    let head = heads.get(0).unwrap();
    let invoke_error = || EvalError::InvalidFunctionCall {
        values: values.to_vec(),
    };

    if let LispVal::Symbol(atom) = head {
        let scope = scope.with_context(atom.clone());

        let (scope, tail) = if head.is_macro() {
            (scope, tail.to_vec())
        } else {
            eval_tail(scope, tail)?
        };

        if atom == "list" {
            return Ok((scope, tail.into()));
        }

        // Internal functions
        if let Some(native_function) = INTERNAL_SYMBOLS_TABLE.get(atom.as_str()) {
            return native_function.call(scope, &tail);
        };

        if let Some(value) = scope.get(atom.as_str()) {
            if let LispVal::Function {
                parameters,
                body,
                applied,
            } = value
            {
                return eval_function(
                    scope.clone(),
                    parameters,
                    body,
                    applied.iter().chain(tail.iter()).cloned().collect(),
                );
            } else {
                return Err(EvalError::InvalidFunctionCall {
                    values: values.to_vec(),
                });
            }
        };

        return Err(EvalError::UnknownIdentifier(atom.clone()));
    };

    if let LispVal::Function {
        parameters,
        body,
        applied,
    } = head
    {
        return eval_function(
            scope.with_context("anonymous".to_string()),
            parameters,
            body,
            applied.iter().chain(tail).cloned().collect(),
        );
    };

    Err(invoke_error())
}

fn eval_tail(scope: Scope, tail: &[LispVal]) -> Result<(Scope, Vec<LispVal>), EvalError> {
    tail.into_iter()
        .try_fold((scope, Vec::<LispVal>::new()), |(scope, mut acc), value| {
            let (scope, value) = eval(scope, &value)?;
            Ok((scope, {
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
        crate::evaluation::eval(
            $crate::evaluation::scope::Scope::default(),
            &parse_it!($expr),
        )
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
        assert_eq!(
            eval_it!("(list (let 'x 10) (+ x 2))"),
            vec![LispVal::Void(), LispVal::Number(12)].into()
        );
    }

    #[test]
    fn test_fold() {
        assert_eq!(eval_it!("(fold '(+) 1 '(1 2 3))"), LispVal::Number(7));
    }

    #[test]
    fn test_map() {
        assert_eq!(
            eval_it!("(map '(+ 2) '(1 2 3))"),
            vec![LispVal::Number(3), LispVal::Number(4), LispVal::Number(5)].into()
        );
    }

    #[test]
    fn test_function_call() {
        assert_eq!(
            eval_it!("(list (let 'add2 '(+ 2)) (map add2 (list 1 2 3)))"),
            vec![
                LispVal::Void(),
                vec![LispVal::Number(3), LispVal::Number(4), LispVal::Number(5)].into()
            ]
            .into()
        );
    }
}
