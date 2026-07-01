use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranspileError {
    #[error("Parse error at line {line}, col {col}: {message}")]
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Unsupported syntax: {0}")]
    UnsupportedSyntax(String),

    #[error("Capability inference failed: {0}")]
    CapabilityInferenceFailed(String),

    #[error("AST walk error: {0}")]
    AstWalkError(String),
}
