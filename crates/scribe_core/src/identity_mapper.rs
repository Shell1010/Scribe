use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

const CACHE_PATH: &str = "scribe_cache.json";

// Only flush to disk at most once every 5 seconds to avoid
// hammering the filesystem on rapid MoveToArea packets.
const FLUSH_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DiskCacheStructure {
    players: HashMap<u32, String>,
    monsters: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    access_levels: HashMap<u32, i32>,
}

#[derive(Debug, Clone)]
pub struct IdentityMapper {
    cache_file: &'static str,
    players: Arc<RwLock<HashMap<u32, String>>>,
    monsters: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
    access_levels: Arc<RwLock<HashMap<u32, i32>>>,
    current_map: Arc<RwLock<String>>,
    // Dirty flag: set by register_* methods, cleared by sync_to_disk.
    // Avoids acquiring read locks + serializing when nothing changed.
    dirty: Arc<AtomicBool>,
    last_flush: Arc<RwLock<Instant>>,
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

        // Only touch disk if the file exists — skip the stat call otherwise.
        if Path::new(CACHE_PATH).exists() {
            if let Ok(mut file) = File::open(CACHE_PATH) {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    if let Ok(decoded) = serde_json::from_str::<DiskCacheStructure>(&content) {
                        loaded_players = decoded.players;
                        loaded_monsters = decoded.monsters;
                        loaded_access_levels = decoded.access_levels;
                    }
                }
            }
        }

        Self {
            cache_file: CACHE_PATH,
            players: Arc::new(RwLock::new(loaded_players)),
            monsters: Arc::new(RwLock::new(loaded_monsters)),
            access_levels: Arc::new(RwLock::new(loaded_access_levels)),
            current_map: Arc::new(RwLock::new(String::new())),
            dirty: Arc::new(AtomicBool::new(false)),
            last_flush: Arc::new(RwLock::new(Instant::now() - FLUSH_INTERVAL)),
        }
    }

    // Batched flush: only writes if dirty AND enough time has elapsed.
    // This turns N writes per MoveToArea into at most 1.
    fn sync_to_disk(&self) {
        if !self.dirty.load(Ordering::Relaxed) {
            return;
        }

        // Rate-limit: skip if we flushed recently.
        {
            let last = self.last_flush.read().unwrap();
            if last.elapsed() < FLUSH_INTERVAL {
                return;
            }
        }

        // Take a snapshot while holding locks briefly, then release
        // before doing the expensive I/O.
        let payload = {
            let p = self.players.read().unwrap();
            let m = self.monsters.read().unwrap();
            let a = self.access_levels.read().unwrap();
            DiskCacheStructure {
                players: p.clone(),
                monsters: m.clone(),
                access_levels: a.clone(),
            }
        };

        if let Ok(serialized) = serde_json::to_string(&payload) {
            if let Ok(mut file) = File::create(self.cache_file) {
                let _ = file.write_all(serialized.as_bytes());
            }
        }

        self.dirty.store(false, Ordering::Relaxed);
        if let Ok(mut last) = self.last_flush.write() {
            *last = Instant::now();
        }
    }

    /// Force-flush on zone change so we don't lose data between rooms.
    fn flush_if_dirty(&self) {
        if !self.dirty.load(Ordering::Relaxed) {
            return;
        }
        let payload = {
            let p = self.players.read().unwrap();
            let m = self.monsters.read().unwrap();
            let a = self.access_levels.read().unwrap();
            DiskCacheStructure {
                players: p.clone(),
                monsters: m.clone(),
                access_levels: a.clone(),
            }
        };
        if let Ok(serialized) = serde_json::to_string(&payload) {
            if let Ok(mut file) = File::create(self.cache_file) {
                let _ = file.write_all(serialized.as_bytes());
            }
        }
        self.dirty.store(false, Ordering::Relaxed);
        if let Ok(mut last) = self.last_flush.write() {
            *last = Instant::now();
        }
    }

    pub fn register_player(&self, id: u32, name: &str) {
        let mut modified = false;
        let name_lower = name.to_lowercase();

        // Early exit: if player already exists with same name, skip write lock.
        {
            if let Ok(p) = self.players.read() {
                if let Some(existing) = p.get(&id) {
                    if *existing == name_lower {
                        // Player already known with correct name — check access level only.
                        if let Ok(a) = self.access_levels.read() {
                            if a.contains_key(&id) {
                                return; // Nothing to do.
                            }
                        }
                    }
                }
            }
        }

        // Need to write — acquire write lock.
        if let Ok(mut p_map) = self.players.write() {
            if p_map.insert(id, name_lower.clone()).is_none()
                || p_map.get(&id).map_or(false, |s| *s != name_lower)
            {
                modified = true;
            }
        }

        if let Ok(mut a_map) = self.access_levels.write() {
            if !a_map.contains_key(&id) {
                a_map.insert(id, 0);
                modified = true;
            }
        }

        if modified {
            self.dirty.store(true, Ordering::Relaxed);
            self.sync_to_disk();
        }
    }

    pub fn register_staff_level(&self, id: u32, access_level: i32) {
        // Quick check: skip if already at this level.
        if let Ok(map) = self.access_levels.read() {
            if map.get(&id).copied() == Some(access_level) {
                return;
            }
        }

        let mut modified = false;
        if let Ok(mut map) = self.access_levels.write() {
            if map.insert(id, access_level).is_none()
                || map.get(&id).copied().unwrap_or(0) != access_level
            {
                modified = true;
            }
        }
        if modified {
            self.dirty.store(true, Ordering::Relaxed);
            self.sync_to_disk();
        }
    }

    // Take &str for id to avoid callers needing to allocate a String.
    pub fn register_monster(&self, map_name: &str, id: &str, name: &str) {
        let mut modified = false;
        let name_lower = name.to_lowercase();
        if let Ok(mut map) = self.monsters.write() {
            let room = map.entry(map_name.to_string()).or_default();
            if room.insert(id.to_string(), name_lower.clone()).is_none()
                || room.get(id).map_or(false, |s| *s != name_lower)
            {
                modified = true;
            }
        }
        if modified {
            self.dirty.store(true, Ordering::Relaxed);
            self.sync_to_disk();
        }
    }

    pub fn set_current_map(&self, map_name: &str) {
        // Force-flush pending writes before switching rooms so we
        // don't lose identity data on crash.
        self.flush_if_dirty();

        if let Ok(mut current) = self.current_map.write() {
            if current.as_str() != map_name {
                *current = map_name.to_string();
            }
        }
    }

    pub fn resolve_actor(&self, actor_token: &str) -> String {
        if actor_token.starts_with("m:") {
            if let (Ok(current), Ok(map)) = (self.current_map.read(), self.monsters.read()) {
                if let Some(room) = map.get(current.as_str()) {
                    if let Some(name) = room.get(actor_token) {
                        return name.clone();
                    }
                }
            }
            return format!("Unknown Monster ({})", actor_token);
        }

        if let Some(id_str) = actor_token.strip_prefix("p:") {
            if let Ok(id) = id_str.parse::<u32>() {
                if let Ok(p_map) = self.players.read() {
                    if let Some(name) = p_map.get(&id) {
                        return name.clone();
                    }
                }
                return format!("Unknown Player ({})", id);
            }
        }

        actor_token.to_string()
    }

    pub fn resolve_actor_typed(&self, actor_token: &str) -> (String, bool) {
        let is_monster = actor_token.starts_with("m:");
        (self.resolve_actor(actor_token), is_monster)
    }
}
