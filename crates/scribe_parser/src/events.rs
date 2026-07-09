use scribe_core::models::combat::StatDetails;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuraTimelineDelta {
    pub action: String,
    pub caster: String,
    pub target: String,
    pub aura_name: String,
    pub value: Option<i32>,
    pub duration: Option<i32>,
    pub aura_type: Option<String>,
    pub is_new: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatTimelineDelta {
    pub target: String,
    pub hp: Option<i32>,
    pub mp: Option<i32>,
    pub shield: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum ScribeEvent {
    CombatTick {
        stats: Vec<StatTimelineDelta>,
        auras: Vec<AuraTimelineDelta>,
    },
    EnemyDamage { caster: String, target: String, damage: i32 },
    PlayerDamage { caster: String, target: String, damage: i32 },
    MonsterReset {
        target: String,
        base_hp: i32,
    },

    Death {
        victim: String,
        killer: String,
    },
    ZoneChange {
        room: String,
    },
    StateChange {
        username: String,
        state: i32,
        entity_id: u32,
        level: i32,
    },

    UserDataInitialized {
        username: String,
        uid: u32,
        access_level: i32,
        class_name: String,
    },

    BossAction {
        caster: String,
        target: String,
        message: String,
        action_type: String,
    },

    StatUpdate {
        stats: StatDetails
    },

    Unknown {
        data: String
    },

    GoldExpGained {
        monster_name: String,
        gold: i32,
        exp: i32,
        bonus_gold: i32,
    },

    PassiveAurasApplied {
        target: String,
        auras: Vec<(String, StatDetails)>
    },

    SkillsLoaded {
        active: Vec<Map<String, Value>>,
        passive: Vec<Map<String, Value>>
    },

    Seia {
        data: Map<String, Value>
    },

    ClassUpdated { uid: u32, class_name: String, category: String, desc: String, mrm: Vec<String> },

    RoomPlayersUpdate { players: Vec<String> },

    ItemDropped {
        item_id: u32,
        item_name: String,
        quantity: u32,
    },

    ItemAdded {
        item_id: u32,
        quantity: u32,
        quantity_now: u32,
    },

    InventoryLoaded {
        items: std::collections::HashMap<u32, String>,
    },

}
