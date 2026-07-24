pub mod combat;
pub mod guild;
pub mod identity;
pub mod state;

use serde::{Deserialize, Serialize};

pub use combat::{
    AddGoldExpPayload, AddItemsPayload, AuraDetails, AuraEvent, AuraPlusPPayload, DropItemPayload,
    MonsterStats, MtlsPayload, PlayerStats, SActPayload, SaraEvent, SarsaEvent, UpdateClassPayload,
};
pub use guild::UpdateGuildPayload;
pub use identity::{LoadInventoryBigPayload, MonsterDefinition, MoveToAreaPayload, PlayerBranch};
pub use state::StateDelta;

use crate::models::combat::StatUpdatePayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitUserDatasPayload {
    pub a: Vec<InitUserDataPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitUserDataPayload {
    pub uid: u32,
    pub data: UserDataDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataDetails {
    #[serde(rename = "strUsername")]
    pub username: String,

    #[serde(rename = "intAccessLevel")]
    pub access_level: i32,

    #[serde(rename = "strClassName")]
    pub class_name: Option<String>,

    #[serde(rename = "intLevel", default)]
    pub level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfsEnvelope {
    pub t: String,
    pub b: SfsBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfsBody {
    pub o: SfsContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
#[allow(clippy::large_enum_variant)]
pub enum SfsContent {
    #[serde(rename = "ct")]
    Combat(combat::CombatPayload),

    #[serde(rename = "uotls")]
    UserState(state::UserStatePayload),

    #[serde(rename = "moveToArea")]
    MoveToArea(identity::MoveToAreaPayload),

    #[serde(rename = "loadInventoryBig")]
    LoadInventoryBig(LoadInventoryBigPayload),

    #[serde(rename = "playerDeath")]
    PlayerDeath(state::PlayerDeathPayload),

    #[serde(rename = "initUserData")]
    InitUserData(InitUserDataPayload),

    #[serde(rename = "initUserDatas")]
    InitUserDatas(InitUserDatasPayload),

    #[serde(rename = "stu")]
    StatUpdate(StatUpdatePayload),

    #[serde(rename = "updateGuild")]
    UpdateGuild(UpdateGuildPayload),

    #[serde(rename = "addGoldExp")]
    AddGoldExp(AddGoldExpPayload),

    #[serde(rename = "aura+p")]
    AuraPlusP(AuraPlusPPayload),

    #[serde(rename = "sAct")]
    SAct(SActPayload),

    #[serde(rename = "seia")]
    Seia(combat::SeiaPayload),

    #[serde(rename = "updateClass")]
    UpdateClass(UpdateClassPayload),

    #[serde(rename = "dropItem")]
    DropItem(DropItemPayload),

    #[serde(rename = "addItems")]
    AddItems(AddItemsPayload),

    #[serde(rename = "mtls")]
    Mtls(MtlsPayload),

    #[serde(rename = "who")]
    Who(WhoPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoPayload {
    #[serde(default)]
    pub users: std::collections::HashMap<String, WhoUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoUser {
    #[serde(rename = "ID")]
    pub id: u32,
    #[serde(rename = "sName")]
    pub name: String,
    #[serde(rename = "sClass")]
    pub class: String,
    #[serde(rename = "iLvl")]
    pub level: i32,
}
