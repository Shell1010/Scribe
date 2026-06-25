use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGuildPayload {
    pub guild: GuildDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildDetails {
    #[serde(rename = "ul")]
    pub user_list: Vec<GuildMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMember {
    #[serde(rename = "ID")]
    pub id: u32,
    
    #[serde(rename = "userName")]
    pub username: String,
}