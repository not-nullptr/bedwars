use crate::{Readable, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct PongResponse {
    pub timestamp: i64,
}
