use crate::parsing::LispVal;

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

fn eval_check_argument_types<'a>(types_list: &'a [&str]) -> impl Fn(&[LispVal]) -> EvalResult + 'a {
    move |values| {
        if values.len() < types_list.len() {
            return Err(EvalError::new(format!(
                "Invalid arguments count, expected {}, got {}",
                types_list.len(),
                values.len()
            )));
        }

        if values.len() > types_list.len() {
            return Err(EvalError::new(format!(
                "Too much arguments, expected {}, got {}",
                types_list.len(),
                values.len()
            )));
        }

        for (i, expected_type_name) in types_list.iter().enumerate() {
            let value_type_name = values.get(i).unwrap().to_type_name();

            if *expected_type_name == "any" {
                continue;
            }

            if *expected_type_name != value_type_name {
                return Err(
                    EvalError::new(format!("Invalid argument type at position {i}, expected {expected_type_name}, got {value_type_name}"))
                );
            }
        }

        Ok(LispVal::Void())
    }
}

fn eval_print(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&["any"])(values)?;
    print!("{}", values.first().unwrap());

    Ok(LispVal::Void())
}

fn eval_math(operation: &str, values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&["number", "number"])(values)?;

    let op = match operation {
        "add" => |a, b| a + b,
        "sub" => |a, b| a - b,
        "mul" => |a, b| a * b,
        "div" => |a, b| a / b,
        _ => unreachable!(),
    };

    let fist = values[0].as_number().unwrap();
    let second = values[1].as_number().unwrap();

    Ok(LispVal::Number(op(fist, second)))
}

fn eval_to_string(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&["number"])(values)?;

    let n = &values[0].as_number().unwrap();
    Ok(LispVal::String(n.to_string()))
}

fn eval_fold(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&["atom", "any", "list"])(values)?;

    let operation = &values[0];
    let initial = &values[1];
    let list = &values[2].as_list().unwrap();

    list.iter().try_fold(initial.clone(), |acc, value| {
        eval(&LispVal::List(vec![operation.clone(), acc, value.clone()]))
    })
}

fn eval_concat_list(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&["list", "list"])(values)?;

    let list_a = &values[0].as_list().unwrap();
    let list_b = &values[1].as_list().unwrap();

    Ok(LispVal::List(
        list_a.iter().chain(list_b.iter()).cloned().collect(),
    ))
}

fn eval_concat_string(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&["string", "string"])(values)?;

    let first = &values[0].as_string().unwrap();
    let second = &values[1].as_string().unwrap();

    Ok(LispVal::String(format!("{}{}", first, second)))
}

fn eval_concat(values: &[LispVal]) -> EvalResult {
    eval_check_argument_types(&["any", "any"])(values)?;

    let first = &values[0];
    let second = &values[1];

    match (first, second) {
        (LispVal::List(_), LispVal::List(_)) => eval_concat_list(values),
        (LispVal::String(_), LispVal::String(_)) => eval_concat_string(values),
        _ => Err(EvalError::new(format!(
            "Invalid argument types, cannot concat {} and {}",
            first.to_type_name(),
            second.to_type_name()
        ))),
    }
}

fn eval_list(values: &[LispVal]) -> EvalResult {
    if values.is_empty() {
        return Ok(LispVal::List(vec![]));
    }

    let head = eval(&values[0])?;
    let tail = values[1..].iter().try_fold(Vec::new(), |mut acc, cur| {
        acc.push(eval(cur)?);
        Ok(acc)
    })?;

    if let LispVal::Symbol(atom) = head {
        return match atom.as_str() {
            "eval" => eval(&tail[0]),
            "print" => eval_print(&tail),
            "add" | "sub" | "mul" | "div" => eval_math(&atom, &tail),
            "concat" => eval_concat(&tail),
            "to_string" => eval_to_string(&tail),
            "fold" => eval_fold(&tail),
            _ => Err(EvalError::new(format!("Unknown identifier \"{atom}\""))),
        };
    };

    Err(EvalError::new(format!(
        "Invalid function call, expected identifier, got {head}",
    )))
}

pub fn eval(expr: &LispVal) -> EvalResult {
    match expr {
        LispVal::List(elements) => eval_list(&elements),
        LispVal::Unevaluated(value) => Ok(*value.clone()),
        _ => Ok(expr.clone()),
    }
}
