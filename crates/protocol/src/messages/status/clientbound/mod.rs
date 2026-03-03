mod pong_response;
mod status_response;

pub use pong_response::*;
pub use status_response::*;

use crate::Writable;

#[derive(Debug, Clone, Writable)]
pub enum ClientboundStatusMessage {
    #[discriminant(0x00)]
    StatusResponse(StatusResponse),

    #[discriminant(0x01)]
    PongResponse(PongResponse),
}
