mod login;

pub use login::*;

use crate::Writable;

#[derive(Debug, Clone, Writable)]
pub enum ClientboundPlayMessage {
    #[discriminant(0x30)]
    Login(Login),
}
