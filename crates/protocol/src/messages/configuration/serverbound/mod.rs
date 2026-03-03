mod acknowledge_finish_configuration;
mod serverbound_known_packs;

pub use acknowledge_finish_configuration::*;
pub use serverbound_known_packs::*;

use crate::Readable;

#[derive(Debug, Clone, Readable)]
pub enum ServerboundConfigurationMessage {
    #[discriminant(0x03)]
    AcknowledgeFinishConfiguration(AcknowledgeFinishConfiguration),

    #[discriminant(0x07)]
    ServerboundKnownPacks(ServerboundKnownPacks),
}
