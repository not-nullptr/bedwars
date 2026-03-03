use thiserror::Error;

#[derive(Debug, Error)]
pub enum RwError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("invalid enum discriminant: {0}")]
    InvalidEnumDiscriminant(u32),

    #[error("serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("NBT error: {0}")]
    Nbt(#[from] fastnbt::error::Error),
}
