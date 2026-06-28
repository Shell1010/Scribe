use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

const CACHE_PATH: &str = "scribe_cache.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DiskCacheStructure {
    players: HashMap<u32, String>,
    monsters: HashMap<String, HashMap<String, String>>,
    #[serde(default)] 
    access_levels: HashMap<u32, i32>, 
}

#[derive(Debug, Clone)]
pub struct IdentityMapper {
    cache_file: String,
    players: Arc<RwLock<HashMap<u32, String>>>,
    monsters: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    access_levels: Arc<RwLock<HashMap<u32, i32>>>,
    current_map: Arc<RwLock<String>>,
}


impl Default for IdentityMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityMapper {
    pub fn new() -> Self {
        let mut loaded_players = HashMap::new();
        let mut loaded_monsters = HashMap::new();
        let mut loaded_access_levels = HashMap::new();

        if Path::new(CACHE_PATH).exists() && let Ok(mut file) = File::open(CACHE_PATH) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() && let Ok(decoded) = serde_json::from_str::<DiskCacheStructure>(&content) {
                loaded_players = decoded.players;
                loaded_monsters = decoded.monsters;
                loaded_access_levels = decoded.access_levels;
            }
            
        }

        Self {
            cache_file: CACHE_PATH.to_string(),
            players: Arc::new(RwLock::new(loaded_players)),
            monsters: Arc::new(RwLock::new(loaded_monsters)),
            access_levels: Arc::new(RwLock::new(loaded_access_levels)),
            current_map: Arc::new(RwLock::new(String::new())),
        }
    }


    fn sync_to_disk(&self) {
        let p_lock = self.players.read().unwrap();
        let m_lock = self.monsters.read().unwrap();
        let a_lock = self.access_levels.read().unwrap();
        
        let payload = DiskCacheStructure {
            players: p_lock.clone(),
            monsters: m_lock.clone(),
            access_levels: a_lock.clone(),
        };

        if let Ok(serialized) = serde_json::to_string_pretty(&payload) && let Ok(mut file) = File::create(&self.cache_file) {
            let _ = file.write_all(serialized.as_bytes());
        }
    }

    pub fn register_player(&self, id: u32, name: &str) {
        let mut modified = false;
        if let Ok(mut p_map) = self.players.write() && (p_map.insert(id, name.to_string()).is_none() || p_map.get(&id).map(|s| s != name).unwrap_or(false)) {
            modified = true;
            
        }

        if let Ok(mut a_map) = self.access_levels.write() && !a_map.contains_key(&id) {
            a_map.insert(id, 0);
            modified = true;
        }

        if modified { self.sync_to_disk(); }
    }

    pub fn register_staff_level(&self, id: u32, access_level: i32) {
        let mut modified = false;
        if let Ok(mut map) = self.access_levels.write() && (map.insert(id, access_level).is_none() || map.get(&id).copied().unwrap_or(0) != access_level) {
            modified = true;
        }
        if modified { self.sync_to_disk(); }
    }

    pub fn register_monster(&self, map_name: &str, id: String, name: &str) {
        let mut modified = false;
        if let Ok(mut map) = self.monsters.write() {
            let room_monsters = map.entry(map_name.to_string()).or_insert_with(HashMap::new);
            if room_monsters.insert(id.clone(), name.to_string()).is_none() || room_monsters.get(&id).map(|s| s != name).unwrap_or(false) {
                modified = true;
            }
        }
        if modified { self.sync_to_disk(); }
    }

    pub fn set_current_map(&self, map_name: &str) {
        if let Ok(mut current) = self.current_map.write() {
            *current = map_name.to_string();
        }
    }
        
        
    pub fn resolve_actor(&self, actor_token: &str) -> String {
        if actor_token.starts_with("m:") {
            if let Ok(current) = self.current_map.read() && let Ok(map) = self.monsters.read() && let Some(room_monsters) = map.get(&*current) && let Some(name) = room_monsters.get(actor_token) {
                return name.clone();
            }
            return format!("Unknown Monster ({})", actor_token);
        }

        if let Some(id_str) = actor_token.strip_prefix("p:") && let Ok(id) = id_str.parse::<u32>() {
            if let Ok(p_map) = self.players.read() && let Some(name) = p_map.get(&id) {
                if let Ok(a_map) = self.access_levels.read() && let Some(&level) = a_map.get(&id) && level > 0 {
                    return name.clone()
                }
                return name.clone()
            }
            return format!("Unknown Player ({})", id);
        }
        
        actor_token.to_string()
    }
}