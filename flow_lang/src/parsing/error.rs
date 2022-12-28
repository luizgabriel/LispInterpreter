use super::LispType;


#[derive(Debug)]
pub struct LispValUnwrapError {
    pub expected: LispType,
    pub got: LispType,
}

impl std::fmt::Display for LispValUnwrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Expected {}, got {}",
            self.expected.to_string(),
            self.got.to_string()
        )
    }
}

impl std::error::Error for LispValUnwrapError {}