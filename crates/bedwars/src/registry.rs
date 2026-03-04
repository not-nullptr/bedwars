use crate::tag;
use protocol::{
    Identifier,
    messages::configuration::{RegistryData, RegistryEntry},
};
use std::{collections::HashMap, path::Path, sync::Arc};

pub const NECESSARY_REGISTRIES: &[Identifier] = &[
    Identifier::const_new("banner_pattern"),
    Identifier::const_new("chat_type"),
    Identifier::const_new("damage_type"),
    Identifier::const_new("dimension_type"),
    Identifier::const_new("instrument"),
    Identifier::const_new("jukebox_song"),
    Identifier::const_new("painting_variant"),
    Identifier::const_new("test_environment"),
    Identifier::const_new("test_instance"),
    Identifier::const_new("timeline"),
    Identifier::const_new("trim_material"),
    Identifier::const_new("trim_pattern"),
    Identifier::const_new("worldgen/biome"),
    Identifier::const_new("cat_variant"),
    Identifier::const_new("chicken_variant"),
    Identifier::const_new("cow_variant"),
    Identifier::const_new("frog_variant"),
    Identifier::const_new("pig_variant"),
    Identifier::const_new("wolf_variant"),
    Identifier::const_new("wolf_sound_variant"),
    Identifier::const_new("zombie_nautilus_variant"),
];

#[derive(Debug, Clone)]
pub struct WithSortOrder<T> {
    sort_order: usize,
    data: T,
}

impl<T> WithSortOrder<T> {
    pub fn new(sort_order: usize, data: T) -> Self {
        Self { sort_order, data }
    }
}

impl<T> std::ops::Deref for WithSortOrder<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for WithSortOrder<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Debug, Clone)]
pub struct SharedRegistryData {
    pub registry_id: Arc<Identifier>,
    pub entries: Arc<[SharedRegistryEntry]>,
}

#[derive(Debug, Clone)]
pub struct SharedRegistryEntry {
    pub entry_id: Arc<Identifier>,
    pub nbt: Option<Arc<fastnbt::Value>>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    registry: Arc<HashMap<Arc<Identifier>, HashMap<Arc<Identifier>, Option<Arc<fastnbt::Value>>>>>,
    tag_map: Arc<HashMap<Arc<Identifier>, Arc<[Arc<Identifier>]>>>,
    sorted_registry: Arc<[SharedRegistryData]>,
    reverse_sorted_registry: Arc<HashMap<Arc<Identifier>, Arc<HashMap<Arc<Identifier>, usize>>>>,
}

impl Registry {
    pub async fn discover<P: AsRef<Path>>(
        root_path: P,
        identifiers: &[Identifier],
    ) -> color_eyre::Result<Self> {
        let mut reverse_sorted_registry = HashMap::new();

        let sorted_registry = protocol::registry_data(root_path.as_ref(), identifiers)
            .await?
            .into_iter()
            .map(|d| {
                let reg_id = Arc::new(d.registry_id);
                let mut outer_entry = HashMap::new();
                let res = SharedRegistryData {
                    registry_id: reg_id.clone(),
                    entries: d
                        .entries
                        .into_iter()
                        .enumerate()
                        .map(|(i, e)| {
                            let entry = SharedRegistryEntry {
                                entry_id: Arc::new(e.entry_id),
                                nbt: e.nbt.map(|v| Arc::new(v)),
                            };

                            outer_entry.insert(entry.entry_id.clone(), i);

                            entry
                        })
                        .collect(),
                };

                reverse_sorted_registry.insert(reg_id.clone(), Arc::new(outer_entry));
                res
            })
            .collect::<Arc<_>>();

        let tag_map: HashMap<_, _> =
            tag::load_tags(&root_path.as_ref().join("minecraft/tags"), "minecraft")?
                .into_iter()
                .map(|(k, v)| (Arc::new(k), v.into_iter().map(Arc::new).collect::<Arc<_>>()))
                .collect();

        let registry: HashMap<_, _> = sorted_registry
            .iter()
            .map(|d| {
                (
                    d.registry_id.clone(),
                    d.entries
                        .iter()
                        .map(|e| (e.entry_id.clone(), e.nbt.clone()))
                        .collect(),
                )
            })
            .collect();

        Ok(Self {
            registry: Arc::new(registry),
            reverse_sorted_registry: Arc::new(reverse_sorted_registry),
            sorted_registry,
            tag_map: Arc::new(tag_map),
        })
    }

    pub fn registry_data(
        &self,
    ) -> Arc<HashMap<Arc<Identifier>, HashMap<Arc<Identifier>, Option<Arc<fastnbt::Value>>>>> {
        self.registry.clone()
    }

    pub fn tag_map(&self) -> Arc<HashMap<Arc<Identifier>, Arc<[Arc<Identifier>]>>> {
        self.tag_map.clone()
    }

    pub fn sorted_registry(&self) -> Arc<[SharedRegistryData]> {
        self.sorted_registry.clone()
    }

    pub fn reverse_sorted_registry(
        &self,
    ) -> Arc<HashMap<Arc<Identifier>, Arc<HashMap<Arc<Identifier>, usize>>>> {
        self.reverse_sorted_registry.clone()
    }
}
