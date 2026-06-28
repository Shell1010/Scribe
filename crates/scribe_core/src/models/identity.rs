use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoveToAreaPayload {
    #[serde(rename = "strMapName", default)]
    pub map_name: String,

    #[serde(rename = "uoBranch", default)]
    pub uo_branch: Vec<PlayerBranch>,
    
    #[serde(default)]
    pub mondef: Vec<MonsterDefinition>,
    
    #[serde(rename = "monBranch", default)]
    pub mon_branch: Vec<MonsterBranch>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterBranch {
    #[serde(rename = "MonMapID")]
    pub mon_map_id: u32,
    
    #[serde(rename = "MonID")]
    pub mon_id: u32,
    
    #[serde(rename = "intHP")]
    pub hp: Option<i32>,
    
    #[serde(rename = "intHPMax")]
    pub max_hp: Option<i32>,
    
    #[serde(rename = "iLvl")]
    pub level: Option<i32>,
}