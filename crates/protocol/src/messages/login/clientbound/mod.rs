mod login_success;

pub use login_success::*;

use crate::Writable;

#[derive(Debug, Clone, Writable)]
pub enum ClientboundLoginMessage {
    #[discriminant(0x02)]
    LoginSuccess(LoginSuccess),
}
