use crate::{Readable, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct PingRequest {
    pub timestamp: i64,
}
