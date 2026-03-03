mod login_acknowledged;
mod login_start;

pub use login_acknowledged::*;
pub use login_start::*;

use crate::Readable;

#[derive(Debug, Clone, Readable)]
pub enum ServerboundLoginMessage {
    #[discriminant(0x00)]
    LoginStart(LoginStart),

    #[discriminant(0x03)]
    LoginAcknowledged(LoginAcknowledged),
}
