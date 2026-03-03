use crate::{Readable, Writable, json::Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Readable, Writable)]
pub struct StatusResponse {
    pub json_response: Json<StatusData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusData {
    pub version: VersionInfo,
    pub players: PlayerInfo,
    pub description: Description,
    pub favicon: Option<String>,
    pub enforces_secure_chat: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub name: String,
    pub protocol: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfo {
    pub max: u32,
    pub online: u32,
    pub sample: Vec<PlayerSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSample {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    pub text: String,
}
