use crate::tag;
use protocol::{Identifier, messages::configuration::RegistryData};
use std::{collections::HashMap, path::Path};

const NECESSARY_REGISTRIES: &[Identifier] = &[Identifier];

#[derive(Debug, Clone)]
pub struct Registry {
    registry: Vec<RegistryData>,
    tag_map: HashMap<Identifier, Vec<Identifier>>,
}

impl Registry {
    pub async fn discover<P: AsRef<Path>>(root_path: P) -> color_eyre::Result<Self> {
        let registry_data = protocol::registry_data(root_path.as_ref()).await?;
        let tag_map = tag::load_tags(&root_path.as_ref().join("minecraft/tags"), "minecraft")?;

        Ok(Self {
            registry: registry_data,
            tag_map,
        })
    }

    pub fn registry_data(&self) -> &[RegistryData] {
        &self.registry
    }

    pub fn tag_map(&self) -> &HashMap<Identifier, Vec<Identifier>> {
        &self.tag_map
    }
}
