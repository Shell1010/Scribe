use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CombatPayload {
    #[serde(default)]
    pub m: HashMap<String, MonsterStats>,
    #[serde(default)]
    pub p: HashMap<String, PlayerStats>,
    #[serde(default)]
    pub a: Vec<AuraEvent>,
    #[serde(default)]
    pub anims: Vec<AnimPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimPayload {
    #[serde(rename = "cInf")]
    pub c_inf: String,
    
    #[serde(rename = "tInf")]
    pub t_inf: String,
    
    #[serde(rename = "animStr")]
    pub anim_str: Option<String>,
    
    #[serde(rename = "strFrame")]
    pub str_frame: Option<String>,
    
    pub fx: Option<String>,
    
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonsterStats {
    #[serde(rename = "intHP")]
    pub hp: Option<i32>,
    #[serde(rename = "intSG")]
    pub shield: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerStats {
    #[serde(rename = "intHP")]
    pub hp: Option<i32>,
    #[serde(rename = "intMP")]
    pub mp: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuraEvent {
    pub cmd: String, // "aura+", "aura++", "aura-"
    #[serde(rename = "tInf")]
    pub target: String, // "p:203887" or "m:1"
    #[serde(rename = "cInf")]
    pub caster: Option<String>,
    
    pub aura: Option<AuraDetails>,
    pub auras: Option<Vec<AuraDetails>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuraDetails {
    #[serde(rename = "nam")]
    pub name: String,
    
    #[serde(rename = "dur")]
    pub duration: Option<i32>,
    
    pub val: Option<i32>,

    #[serde(rename = "t")]
    pub aura_type: Option<String>,

    #[serde(rename = "isNew")]
    pub is_new: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatUpdatePayload {
    pub sta: HashMap<String, f64>,
}