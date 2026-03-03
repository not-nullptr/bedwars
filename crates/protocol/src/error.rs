use thiserror::Error;

#[derive(Debug, Error)]
pub enum RwError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("invalid enum discriminant: {0}")]
    InvalidEnumDiscriminant(u32),
}
