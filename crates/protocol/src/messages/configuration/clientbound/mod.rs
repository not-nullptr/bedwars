mod clientbound_known_packs;
mod finish_configuration;
mod registry_data;
mod update_tags;

pub use clientbound_known_packs::*;
pub use finish_configuration::*;
pub use registry_data::*;
pub use update_tags::*;

use crate::Writable;

#[derive(Debug, Clone, Writable)]
pub enum ClientboundConfigurationMessage {
    #[discriminant(0x03)]
    FinishConfiguration(FinishConfiguration),

    #[discriminant(0x07)]
    RegistryData(RegistryData),

    #[discriminant(0x0D)]
    UpdateTags(UpdateTags),

    #[discriminant(0x0E)]
    ClientboundKnownPacks(ClientboundKnownPacks),
}
