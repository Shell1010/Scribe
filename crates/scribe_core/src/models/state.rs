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
    pub level: Option<i32>,
    
    #[serde(rename = "intHP")]
    pub hp: Option<i32>,
    
    #[serde(rename = "intHPMax")]
    pub max_hp: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDelta {
    #[serde(rename = "entID")]
    pub ent_id: Option<u32>,
    
    #[serde(rename = "strUsername")]
    pub username: Option<String>,
    
    #[serde(rename = "intLevel")]
    pub level: Option<i32>,
    
    #[serde(rename = "intState")]
    pub int_state: Option<i32>, // 1: Alive, 2: In Combat, 0: Dead

    #[serde(rename = "intHP")]
    pub hp: Option<i32>,
    
    #[serde(rename = "intHPMax")]
    pub max_hp: Option<i32>,

    #[serde(rename = "intMP")]
    pub mp: Option<i32>,
    
    #[serde(rename = "intMPMax")]
    pub max_mp: Option<i32>,

    pub tx: Option<f64>,
    pub ty: Option<f64>,
    pub afk: Option<bool>,
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