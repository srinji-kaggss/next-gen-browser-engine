use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranspileError {
    #[error("Parse error at line {line}, col {col}: {message}")]
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },
}
