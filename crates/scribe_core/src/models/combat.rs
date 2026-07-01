use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddItemsPayload {
    pub items: std::collections::HashMap<String, AddItemDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddItemDetails {
    #[serde(rename = "iQty")]
    pub i_qty: u32,
    #[serde(rename = "iQtyNow")]
    pub i_qty_now: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropItemPayload {
    pub items: std::collections::HashMap<String, DropItemDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropItemDetails {
    #[serde(rename = "ItemID")]
    pub item_id: u32,
    #[serde(rename = "sName")]
    pub s_name: String,
    #[serde(rename = "iQty")]
    pub i_qty: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddGoldExpPayload {
    #[serde(rename = "intGold")] pub gold: i32,
    #[serde(rename = "intExp")] pub exp: i32,
    #[serde(rename = "bonusGold", default)] pub bonus_gold: i32,
    #[serde(rename = "typ")] pub typ: String,
    pub id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuraPlusPPayload {
    #[serde(rename = "tInf")] pub target_info: String,
    pub auras: Vec<PassiveAura>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveAura {
    pub nam: String,
    #[serde(default)] pub e: Vec<AuraEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuraEffect {
    pub sta: String,
    pub val: f64,
    pub typ: String,
}

impl PassiveAura {
    pub fn to_stat_details(&self) -> StatDetails {
        let mut stats = StatDetails::default();
        for effect in &self.e {
            match effect.sta.as_str() {
                "STR" => stats.total_str = Some(effect.val),
                "DEX" => stats.total_dex = Some(effect.val),
                "INT" => stats.total_int = Some(effect.val),
                "END" => stats.total_end = Some(effect.val),
                "WIS" => stats.total_wis = Some(effect.val),
                "LCK" => stats.total_lck = Some(effect.val),
                
                "ap" => stats.attack_power = Some(effect.val),
                "sp" => stats.spell_power = Some(effect.val),
                "thi" => stats.hit_chance = Some(effect.val),
                "tcr" => stats.crit_rate = Some(effect.val),
                "scm" => stats.crit_mod = Some(effect.val),
                "tha" => stats.haste = Some(effect.val),
                "dsh" => stats.dash = Some(effect.val),
                
                "cao" => stats.damage_boost_all = Some(effect.val),
                "cpo" => stats.physical_boost = Some(effect.val),
                "cmo" => stats.magic_boost = Some(effect.val),
                "cdo" => stats.dot_boost = Some(effect.val),
                "cho" => stats.heal_boost = Some(effect.val),
                
                "cai" => stats.damage_intake = Some(effect.val),
                "cpi" => stats.physical_intake = Some(effect.val),
                "cmi" => stats.magic_intake = Some(effect.val),
                "cdi" => stats.dot_intake = Some(effect.val),
                "chi" => stats.healing_intake = Some(effect.val),
                
                "tdo" => stats.dodge_chance = Some(effect.val),
                "cmc" => stats.mana_consumption = Some(effect.val),
                "shb" => stats.health_boost = Some(effect.val),
                "smb" => stats.mana_boost = Some(effect.val),
                _ => {}
            }
        }
        stats
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SActPayload {
    pub actions: SActActions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SActActions {
    #[serde(default)]
    pub active: Vec<serde_json::Map<String, Value>>,
    
    #[serde(default)]
    pub passive: Vec<serde_json::Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeiaPayload {
    #[serde(default)]
    pub o: serde_json::Map<String, Value>,
    #[serde(default, rename = "iRes")]
    pub i_res: Option<i32>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClassPayload {
    pub uid: u32,
    #[serde(rename = "sClassName")] pub class_name: String,
    #[serde(rename = "sClassCat")] pub class_cat: String,
    #[serde(rename = "iCP", default)] pub cp: i32,
    #[serde(rename = "sDesc", default)] pub desc: String,
    #[serde(rename = "aMRM", default)] pub mrm: Vec<String>,
}


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
    #[serde(default)]
    pub sarsa: Vec<SarsaEvent>,
    #[serde(default)]
    pub sara: Vec<SaraEvent>,
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
    pub cmd: String,
    #[serde(rename = "tInf")]
    pub target: String,
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

impl StatDetails {
    pub fn to_formatted_list(&self) -> Vec<String> {
        let mut changed = Vec::new();
        let mut add = |name: &str, val: Option<f64>| {
            if let Some(v) = val { changed.push(format!("{}: {}", name, v)); }
        };
    
        add("STR", self.total_str);
        add("INT", self.total_int);
        add("END", self.total_end);
        add("WIS", self.total_wis);
        add("DEX", self.total_dex);
        add("LUK", self.total_lck);
        
        add("Attack Power", self.attack_power);
        add("Spell Power", self.spell_power);
        add("All Out", self.damage_boost_all);
        add("Physical Out", self.physical_boost);
        add("Magic Out", self.magic_boost);
        add("Dot Out", self.dot_boost);
        add("Heal Out", self.heal_boost);
        add("All In", self.damage_intake);
        add("Physical In", self.physical_intake);
        add("Magic In", self.magic_intake);
        add("Dot In", self.dot_intake);
        add("Heal In", self.healing_intake);
        
        add("Mana Consumption", self.mana_consumption);
        add("Crit Chance", self.crit_rate);
        add("Dodge", self.dodge_chance);
        add("Haste", self.haste);
        add("Dash", self.dash);
        add("Health Boost", self.health_boost);
        add("Mana Boost", self.mana_boost);
        add("Hit Chance", self.hit_chance);
        add("Crit Mod", self.crit_mod);
    
        changed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarsaEvent {
    #[serde(rename = "cInf")]
    pub c_inf: String,
    pub a: Vec<SarsaAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarsaAction {
    #[serde(rename = "type")]
    pub action_type: Option<String>,
    pub hp: i32,
    #[serde(rename = "tInf")]
    pub t_inf: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaraEvent {
    #[serde(rename = "actionResult")]
    pub action_result: SaraActionResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaraActionResult {
    #[serde(rename = "type")]
    pub action_type: Option<String>,
    pub hp: i32,
    #[serde(rename = "cInf")]
    pub c_inf: String,
    #[serde(rename = "tInf")]
    pub t_inf: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsPayload {
    pub id: i32,
    pub o: MtlsStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsStats {
    #[serde(rename = "intState")]
    pub int_state: Option<i32>,
    #[serde(rename = "intHP")]
    pub int_hp: i32,
    #[serde(rename = "intMP")]
    pub int_mp: Option<i32>,
}