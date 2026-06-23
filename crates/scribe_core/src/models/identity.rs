use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoveToAreaPayload {
    #[serde(default)]
    pub uoBranch: Vec<PlayerBranch>,
    #[serde(default)]
    pub mondef: Vec<MonsterDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBranch {
    #[serde(rename = "entID")]
    pub ent_id: Option<u32>,
    #[serde(rename = "strUsername")]
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterDefinition {
    #[serde(rename = "MonID")]
    pub mon_id: u32,
    #[serde(rename = "strMonName")]
    pub mon_name: String,
}


