mod ping_request;
mod status_request;

pub use ping_request::*;
pub use status_request::*;

use crate::Readable;

#[derive(Debug, Clone, Readable)]
pub enum ServerboundStatusMessage {
    #[discriminant(0x00)]
    StatusRequest(StatusRequest),

    #[discriminant(0x01)]
    PingRequest(PingRequest),
}
