pub mod combat;
pub mod identity;
pub mod state;

use serde::{Deserialize, Serialize};


pub use combat::{MonsterStats, PlayerStats, AuraEvent, AuraDetails};
pub use identity::{MoveToAreaPayload, PlayerBranch, MonsterDefinition};
pub use state::StateDelta;

use crate::models::combat::StatUpdatePayload;


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
pub enum SfsContent {
    #[serde(rename = "ct")]
    Combat(combat::CombatPayload),
    
    #[serde(rename = "uotls")]
    UserState(state::UserStatePayload),
    
    #[serde(rename = "moveToArea")]
    MoveToArea(identity::MoveToAreaPayload),

    #[serde(rename = "playerDeath")]
    PlayerDeath(state::PlayerDeathPayload),
    
    #[serde(rename = "initUserData")]
    InitUserData(InitUserDataPayload),

    #[serde(rename = "stu")]
    StatUpdate(StatUpdatePayload),
}