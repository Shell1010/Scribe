use std::fmt::Display;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};


// Speaper
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Action {
    Taunt,
    Zone,
    Decay,
    Quixotic
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Trigger {
    Message(String),
    HP {
        value: u32,
        timer: u32,
    },
    Timer(u32),
    Aura {
        name: String,
        timer: u32
    },
    LoopAura {
        name: String,
        roles: Vec<String>,
        timer: u32,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MechanicStep {
    pub trigger: Trigger,
    pub role: String,
    pub action: Action,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReactiveStep {
    pub trigger: Trigger,
    pub role: String,
    pub action: Action,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Strategy {
    pub name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening: Option<Vec<MechanicStep>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle: Option<Vec<MechanicStep>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reactives: Option<Vec<ReactiveStep>>,

    pub roles: Vec<String>,
}


pub type BossRegistry = HashMap<String, Vec<Strategy>>;
