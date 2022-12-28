use std::collections::HashMap;
use crate::parsing::{LispVal, LispType};
use lazy_static::lazy_static;

#[derive(Debug)]
pub struct EvalError {
    reason: String,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EvalError: {}", self.reason)
    }
}

impl std::error::Error for EvalError {
    fn description(&self) -> &str {
        &self.reason
    }
}

impl EvalError {
    fn new(reason: String) -> Self {
        Self { reason }
    }
}

type EvalResult<T = LispVal> = Result<T, EvalError>;


fn eval_check_argument_types<'a>(types_list: &'a [LispType]) -> impl Fn(&[LispVal]) -> EvalResult + 'a {
    move |values| {
        if values.len() < types_list.len() {
            return Err(EvalError::new(format!(
                "Invalid arguments count, expected `{}`, got `{}`",
                types_list.len(),
                values.len()
            )));
        }

        if values.len() > types_list.len() {
            return Err(EvalError::new(format!(
                "Too much arguments, expected `{}`, got `{}`",
                types_list.len(),
                values.len()
            )));
        }

        for (i, expected_type) in types_list.iter().enumerate() {
            let value_type = values.get(i).unwrap().to_type();

            if *expected_type == LispType::Any {
                continue;
            }

            if *expected_type != value_type {
                return Err(
                    EvalError::new(format!("Invalid argument type at position `{}`, expected `{}`, got `{}`", i, expected_type, value_type))
                );
            }
        }

        Ok(LispVal::Void())
    }
}

fn eval_print(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&[LispType::Any])(values)?;
    print!("{}", values.first().unwrap());

    Ok(LispVal::Void())
}

fn eval_math<F : Fn(i64, i64) -> i64>(operation: F) -> impl Fn(&[LispVal]) -> EvalResult {
    move |values| {
        eval_check_argument_types(&[LispType::Number, LispType::Number])(values)?;

        let fist = values[0].as_number().unwrap();
        let second = values[1].as_number().unwrap();
    
        Ok(LispVal::Number(operation(fist, second)))
    }
}

fn eval_to_string(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&[LispType::Number])(values)?;

    let n = &values[0].as_number().unwrap();
    Ok(LispVal::String(n.to_string()))
}

fn eval_fold(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&[LispType::Symbol, LispType::Any, LispType::List])(values)?;

    let operation = &values[0];
    let initial = &values[1];
    let list = &values[2].as_list().unwrap();

    list.iter().try_fold(initial.clone(), |acc, value| {
        eval(&LispVal::List(vec![operation.clone(), acc, value.clone()]))
    })
}

fn eval_concat_list(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&[LispType::List, LispType::List])(values)?;

    let list_a = &values[0].as_list().unwrap();
    let list_b = &values[1].as_list().unwrap();

    Ok(LispVal::List(
        list_a.iter().chain(list_b.iter()).cloned().collect(),
    ))
}

fn eval_concat_string(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&[LispType::String, LispType::String])(values)?;

    let first = &values[0].as_string().unwrap();
    let second = &values[1].as_string().unwrap();

    Ok(LispVal::String(format!("{}{}", first, second)))
}

fn eval_concat(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&[LispType::Any, LispType::Any])(values)?;

    let first = &values[0];
    let second = &values[1];

    match (first, second) {
        (LispVal::List(_), LispVal::List(_)) => eval_concat_list(values),
        (LispVal::String(_), LispVal::String(_)) => eval_concat_string(values),
        _ => Err(EvalError::new(format!(
            "Invalid argument types, cannot concat `{}` and `{}`",
            first.to_type(),
            second.to_type()
        ))),
    }
}

fn eval_unevaluated(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&[LispType::Any])(values)?;
    eval(&values[0])
}

lazy_static! {
    static ref SYMBOLS_TABLE: HashMap::<&'static str, Box<dyn Fn(&[LispVal]) -> EvalResult + Sync>>  = {
        let mut s = HashMap::<&'static str, Box<dyn Fn(&[LispVal]) -> EvalResult + Sync>>::new();
        s.insert("eval", Box::new(eval_unevaluated));
        s.insert("print", Box::new(eval_print));
        s.insert("concat", Box::new(eval_concat));
        s.insert("to_string", Box::new(eval_to_string));
        s.insert("fold", Box::new(eval_fold));
        s.insert("+", Box::new(eval_math(|a, b| a + b)));
        s.insert("-", Box::new(eval_math(|a, b| a - b)));
        s.insert("*", Box::new(eval_math(|a, b| a * b)));
        s.insert("/", Box::new(eval_math(|a, b| a / b)));
        s.insert("%", Box::new(eval_math(|a, b| a % b)));
        s
    };
}

fn eval_list(values: &[LispVal]) -> EvalResult {
    if values.is_empty() {
        return Ok(LispVal::List(vec![]));
    }

    let (head, tail) = values.split_first().unwrap();

    if let LispVal::Symbol(atom) = head {
        return match SYMBOLS_TABLE.get(atom.as_str()) {
            Some(f) => f(&tail.iter().map(eval).try_collect::<Vec<LispVal>>()?),
            None => Err(EvalError::new(format!("Unknown identifier `{}`", atom))),
        };
    };

    let correct_expr = LispVal::Unevaluated(Box::new(LispVal::List(values.iter().cloned().collect())));
    Err(EvalError::new(format!(
        "Invalid function call, expected identifier, got `{}`. \nIs this supposed to be a list? If so, use `{}`",
        head,
        correct_expr
    )))
}

pub fn eval(expr: &LispVal) -> EvalResult {
    match expr {
        LispVal::List(elements) => eval_list(elements),
        LispVal::Unevaluated(value) => Ok(*value.clone()),
        _ => Ok(expr.clone()),
    }
}
