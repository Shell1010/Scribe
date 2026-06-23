use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatePayload {
    #[serde(rename = "unm")]
    pub username: String,
    #[serde(rename = "o")]
    pub state_data: StateDelta,
    
    #[serde(rename = "entID")]
    pub ent_id: Option<u32>,
    
    #[serde(rename = "intLevel")]
    pub level: i32,
    
    #[serde(rename = "intHP")]
    pub hp: i32,
    
    #[serde(rename = "intHPMax")]
    pub max_hp: i32,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDelta {
    #[serde(rename = "intState")]
    pub int_state: i32, // 1: Alive, 2: In Combat, 0: Dead
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDeathPayload {
    #[serde(rename = "userID")]
    pub user_id: u32,
    #[serde(rename = "did")]
    pub destroyer_id: u32,
    #[serde(rename = "entType")]
    pub ent_type: String,
}