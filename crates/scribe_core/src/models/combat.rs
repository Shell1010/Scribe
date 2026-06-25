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
    pub sta: StatDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatDetails {
    #[serde(rename = "_STR")]
    pub base_str: Option<f64>,
    #[serde(rename = "_DEX")]
    pub base_dex: Option<f64>,
    #[serde(rename = "_INT")]
    pub base_int: Option<f64>,
    #[serde(rename = "_END")]
    pub base_end: Option<f64>,
    #[serde(rename = "_WIS")]
    pub base_wis: Option<f64>,
    #[serde(rename = "_LCK")]
    pub base_lck: Option<f64>,

    #[serde(rename = "$STR")]
    pub total_str: Option<f64>,
    #[serde(rename = "$DEX")]
    pub total_dex: Option<f64>,
    #[serde(rename = "$INT")]
    pub total_int: Option<f64>,
    #[serde(rename = "$END")]
    pub total_end: Option<f64>,
    #[serde(rename = "$WIS")]
    pub total_wis: Option<f64>,
    #[serde(rename = "$LCK")]
    pub total_lck: Option<f64>,

    #[serde(rename = "$ap")]
    pub attack_power: Option<f64>,
    #[serde(rename = "$sp")]
    pub spell_power: Option<f64>,
    #[serde(rename = "$thi")]
    pub hit_chance: Option<f64>,
    #[serde(rename = "$tcr")]
    pub crit_rate: Option<f64>,
    #[serde(rename = "$scm")]
    pub crit_mod: Option<f64>,
    #[serde(rename = "$tha")]
    pub haste: Option<f64>,
    #[serde(rename = "$dsh")]
    pub dash: Option<f64>,

    #[serde(rename = "$cao")]
    pub damage_boost_all: Option<f64>,
    #[serde(rename = "$cpo")]
    pub physical_boost: Option<f64>,
    #[serde(rename = "$cmo")]
    pub magic_boost: Option<f64>,
    #[serde(rename = "$cdo")]
    pub dot_boost: Option<f64>,
    #[serde(rename = "$cho")]
    pub heal_boost: Option<f64>,


    #[serde(rename = "$cai")]
    pub damage_intake: Option<f64>,
    #[serde(rename = "$cpi")]
    pub physical_intake: Option<f64>,
    #[serde(rename = "$cmi")]
    pub magic_intake: Option<f64>,
    #[serde(rename = "$cdi")]
    pub dot_intake: Option<f64>,
    #[serde(rename = "$chi")]
    pub healing_intake: Option<f64>,

    #[serde(rename = "$tdo")]
    pub dodge_chance: Option<f64>,
    #[serde(rename = "$cmc")]
    pub mana_consumption: Option<f64>,
    
    #[serde(rename = "$shb")]
    pub health_boost: Option<f64>,
    #[serde(rename = "$smb")]
    pub mana_boost: Option<f64>,
}