use crate::{
    Identifier,
    messages::configuration::{RegistryData, RegistryEntry},
};
use fastnbt::{IntArray, LongArray};
use std::{collections::HashMap, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryDataError {
    #[error("failed to read registry data from path: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("failed to parse registry data from path: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("invalid json structure in registry entry")]
    InvalidJson,
}

pub async fn registry_data<P: AsRef<Path>>(
    data_dir: P,
    identifiers: &[Identifier],
) -> Result<Vec<RegistryData>, RegistryDataError> {
    let path = data_dir.as_ref();
    // let mut dirs = tokio::fs::read_dir(path).await?;
    let mut handles = Vec::new();

    // while let Some(entry) = dirs.next_entry().await? {
    //     // if it's not a dir, skip it
    //     if !entry.file_type().await?.is_dir() {
    //         continue;
    //     }

    //     handles.push(tokio::spawn(read_registry(entry)));
    // }

    for ident in identifiers {
        let ident = ident.clone();
        let path = path.to_path_buf();
        handles.push(tokio::spawn(read_registry(path, ident)));
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await??);
    }

    Ok(results)
}

async fn read_registry<P: AsRef<Path>>(
    path: P,
    identifier: Identifier,
) -> Result<RegistryData, RegistryDataError> {
    let path = path
        .as_ref()
        .join(&*identifier.namespace)
        .join(&*identifier.value);

    tracing::info!(path = %path.display(), "reading registry data for identifier");

    let mut entries = tokio::fs::read_dir(path).await?;

    let mut data = RegistryData {
        registry_id: identifier.clone(),
        entries: Vec::new(),
    };

    while let Some(file) = entries.next_entry().await? {
        let kind = file.file_type().await?;
        if kind.is_dir() {
            tracing::error!("invalid base pack!");
            continue;
        }

        let path = file.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            tracing::error!(path = %path.display(), "skipping non-json file in registry directory");
            continue;
        }

        let file_stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        let nbt = match get_nbt(&path).await {
            Ok(nbt) => nbt,
            Err(e) => {
                tracing::error!(%e, path = %path.display(), "failed to convert registry entry json to nbt");
                continue;
            }
        };

        data.entries.push(RegistryEntry {
            entry_id: Identifier::with_namespace(identifier.namespace.clone(), file_stem),
            nbt: Some(nbt),
        });
    }

    Ok(data)
}

async fn get_nbt<P>(path: P) -> Result<fastnbt::Value, RegistryDataError>
where
    P: AsRef<Path>,
{
    // convert json to nbt
    let json = tokio::fs::read_to_string(&path).await?;
    let raw: serde_json::Value = serde_json::from_str(&json)?;
    let serde_json::Value::Object(obj) = raw else {
        tracing::error!(path = %path.as_ref().display(), "expected json object at root of registry entry");
        return Err(RegistryDataError::InvalidJson);
    };

    let mut compound = fastnbt::Value::Compound(HashMap::new());
    // recurse_construct_nbt(&mut compound, obj)?;

    if let Err(e) = recurse_construct_nbt(&mut compound, obj) {
        tracing::error!(%e, path = %path.as_ref().display(), "failed to construct nbt from json");
        return Err(e.into());
    }

    Ok(compound)
}

fn recurse_construct_nbt(
    compound: &mut fastnbt::Value,
    obj: serde_json::Map<String, serde_json::Value>,
) -> Result<(), RegistryDataError> {
    let fastnbt::Value::Compound(map) = compound else {
        return Err(RegistryDataError::InvalidJson);
    };

    for (k, v) in obj {
        map.insert(k, json_to_nbt(&v)?);
    }

    Ok(())
}

fn json_to_nbt(value: &serde_json::Value) -> Result<fastnbt::Value, RegistryDataError> {
    match value {
        serde_json::Value::Null => Err(RegistryDataError::InvalidJson),
        serde_json::Value::Bool(b) => Ok(fastnbt::Value::Byte(if *b { 1 } else { 0 })),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if let Ok(i32v) = i32::try_from(i) {
                    Ok(fastnbt::Value::Int(i32v))
                } else {
                    Ok(fastnbt::Value::Long(i))
                }
            } else if let Some(u) = n.as_u64() {
                if u <= i32::MAX as u64 {
                    Ok(fastnbt::Value::Int(u as i32))
                } else if u <= i64::MAX as u64 {
                    Ok(fastnbt::Value::Long(u as i64))
                } else {
                    Err(RegistryDataError::InvalidJson)
                }
            } else if let Some(f) = n.as_f64() {
                Ok(fastnbt::Value::Double(f))
            } else {
                Err(RegistryDataError::InvalidJson)
            }
        }
        serde_json::Value::String(s) => Ok(fastnbt::Value::String(s.clone())),
        serde_json::Value::Array(arr) => convert_array(arr),
        serde_json::Value::Object(map) => {
            let mut nested = fastnbt::Value::Compound(HashMap::new());
            recurse_construct_nbt(&mut nested, map.clone())?;
            Ok(nested)
        }
    }
}

fn convert_array(arr: &[serde_json::Value]) -> Result<fastnbt::Value, RegistryDataError> {
    if arr.is_empty() {
        return Ok(fastnbt::Value::List(Vec::new()));
    }

    if arr
        .iter()
        .all(|v| v.as_i64().is_some() || v.as_u64().is_some())
    {
        let mut ints = Vec::with_capacity(arr.len());
        let mut longs = Vec::with_capacity(arr.len());
        let mut all_i32 = true;

        for v in arr {
            if let Some(i) = v.as_i64() {
                if let Ok(i32v) = i32::try_from(i) {
                    ints.push(i32v);
                    longs.push(i);
                } else {
                    all_i32 = false;
                    longs.push(i);
                }
            } else if let Some(u) = v.as_u64() {
                if u <= i32::MAX as u64 {
                    ints.push(u as i32);
                    longs.push(u as i64);
                } else if u <= i64::MAX as u64 {
                    all_i32 = false;
                    longs.push(u as i64);
                } else {
                    return Err(RegistryDataError::InvalidJson);
                }
            }
        }

        return if all_i32 {
            Ok(fastnbt::Value::IntArray(IntArray::new(ints)))
        } else {
            Ok(fastnbt::Value::LongArray(LongArray::new(longs)))
        };
    }

    let mut list = Vec::with_capacity(arr.len());
    for v in arr {
        list.push(json_to_nbt(v)?);
    }
    Ok(fastnbt::Value::List(list))
}
