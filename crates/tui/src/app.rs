use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::Write;
use scribe_parser::events::{ScribeEvent, AuraTimelineDelta};
use serde_json::{Map, Value};


const MAX_HISTORY: usize = 1000;

#[derive(Debug, Clone)]
pub struct EntityState {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub shield: i32,
    pub auras: Vec<String>,
    pub last_action: String,
}

impl EntityState {
    pub fn new(name: String) -> Self {
        Self { name, hp: 0, max_hp: 0, mp: 0, shield: 0, auras: Vec::new(), last_action: String::new() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClassInfo {
    pub name: String,
    pub category: String,
    pub desc: String,
    pub mrm: Vec<String>,
    pub active_skills: Vec<Map<String, Value>>,
    pub passive_skills: Vec<Map<String, Value>>,
}



pub struct App {
    pub active_tab: usize,
    pub tabs: Vec<&'static str>,
    pub scroll_output_y: u16,
    pub scroll_output_x: u16,
    pub scroll_class_x: u16,
    pub scroll_class_y: u16,
    
    pub snap: bool,

    pub entities: HashMap<String, EntityState>,
    pub system_log: VecDeque<String>,
    pub last_vitals: HashMap<String, (i32, i32, i32)>,
    recent_events: VecDeque<ScribeEvent>, 
    log_file: Option<File>,
    pub should_quit: bool,
    pub users_in_room: Vec<String>,

    pub class_info: ClassInfo,
    pub selected_skill_index: usize,
        
}

impl App {
    pub fn new(log_path: Option<&str>) -> Self {
        let log_file = log_path.map(|path| {
            OpenOptions::new().create(true).append(true).open(path).expect("Failed to open log file")
        });

        Self {
            active_tab: 0,
            tabs: vec!["Output Log", "Entity Tracker (WIP)"],
            scroll_output_y: 0,
            scroll_output_x: 0,
            scroll_class_x: 0,
            scroll_class_y: 0,
            entities: HashMap::new(),
            system_log: VecDeque::with_capacity(1000),
            last_vitals: HashMap::new(),
            recent_events: VecDeque::with_capacity(1000),
            log_file,
            should_quit: false,
            users_in_room: Vec::new(),
            snap: false,
            class_info: ClassInfo::default(),
            selected_skill_index: 0,
        }
    }

    pub fn snap_to_bottom(&mut self) {
        self.snap = true;
    }
        

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        if self.active_tab > 0 {
            self.active_tab -= 1;
        } else {
            self.active_tab = self.tabs.len() - 1;
        }
    }

    fn push_log(&mut self, message: String) {
        if let Some(file) = &mut self.log_file {
            let _ = writeln!(file, "{}", message);
        }
        if self.system_log.len() >= MAX_HISTORY {
            self.system_log.pop_front();
        }
        self.system_log.push_back(message);
        self.snap = true;
    }

    fn format_aura_metadata(aura: &AuraTimelineDelta) -> String {
        let mut parts = Vec::new();
        if let Some(d) = aura.duration { parts.push(format!("Dur: {}s", d)); }
        if let Some(v) = aura.value { parts.push(format!("Val: {}", v)); }
        if parts.is_empty() { String::new() } else { format!(" [{}]", parts.join(" | ")) }
    }

    pub fn handle_event(&mut self, event: ScribeEvent) {
        if self.recent_events.len() >= MAX_HISTORY {
            self.recent_events.pop_front();
        }
        self.recent_events.push_back(event.clone());

        match event {
            ScribeEvent::RoomPlayersUpdate { players } => {
                self.users_in_room = players;
            }

            ScribeEvent::GoldExpGained { monster_name, gold, exp, bonus_gold } => {
                let bonus = if bonus_gold > 0 { format!(" (+{} bonus)", bonus_gold) } else { "".to_string() };
                self.push_log(format!("  -> [Loot] Killed {}: {} Gold{} | {} Exp", monster_name, gold, bonus, exp));
            }

            ScribeEvent::PassiveAurasApplied { target, auras } => {
                for (name, details) in auras {
                    let readable_stats = details.to_formatted_list();
                    let stats_log = if readable_stats.is_empty() { 
                        "".to_string() 
                    } else { 
                        format!(" ({})", readable_stats.join(", ")) 
                    };
                    
                    self.push_log(format!("  -> [Passives] {} applied to {}{}", name, target, stats_log));
            
                    if let Ok(value) = serde_json::to_value(details) && let Ok(mut stats_map) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(value) {
                        stats_map.retain(|_, v| !v.is_null());
        
                        if !readable_stats.is_empty() {
                            stats_map.insert(
                                "stats".to_string(), 
                                serde_json::Value::String(readable_stats.join(", "))
                            );
                        }

                        if let Some(skill) = self.class_info.passive_skills.iter_mut().find(|s| {
                            s.get("nam").and_then(|v| v.as_str()) == Some(name.as_str())
                        }) {
                            skill.extend(stats_map);
                        } else {
                            stats_map.insert("nam".to_string(), serde_json::Value::String(name.clone()));
                            self.class_info.passive_skills.push(stats_map);
                        }
                        
                    }
                }
            }

            ScribeEvent::ClassUpdated { class_name, category, desc, mrm } => {
                self.class_info.name = class_name.clone();
                self.class_info.category = category.clone();
                self.class_info.desc = desc.clone();
                self.class_info.mrm = mrm.clone();
                self.selected_skill_index = 0;
                
                self.push_log(format!("\n=== Class Switched: {} ({}) ===", class_name, category));
            }


            ScribeEvent::Seia { data } => {
                if self.class_info.active_skills.len() > 5 {
                    self.class_info.active_skills[5] = data;
                }
            }

            ScribeEvent::SkillsLoaded { active, passive } => {
                self.class_info.active_skills = active;
                if self.class_info.passive_skills.is_empty() {
                    self.class_info.passive_skills = passive;
                } else {
                    for (i, skill) in passive.iter().enumerate() {
                        self.class_info.passive_skills[i].extend(skill.clone());
                    }
                }
                self.selected_skill_index = 0;
                
                self.push_log(format!("  -> Loaded {} Active & {} Passive Skills", 
                    self.class_info.active_skills.len(), 
                    self.class_info.passive_skills.len()
                ));
                            
            }
                        
            ScribeEvent::UserDataInitialized { username, uid, access_level, class_name } => {
                let class_str = class_name;
                self.push_log(format!("  -> User data initialized: {} (UID: {}, Access Level: {}, Class: {})", username, uid, access_level, class_str));
            }

            ScribeEvent::BossAction { caster, target, message, action_type } => {
                let msg_part = if message.is_empty() { String::new() } else { format!(" | msg: {}", message) };
                self.push_log(format!("  -> [Boss] {} >> {} : ({}){}", caster, target, action_type, msg_part));
            }

            ScribeEvent::StatUpdate { stats } => {
                let mut changed = Vec::new();
                let mut add = |name: &str, val: Option<f64>| {
                    if let Some(v) = val { changed.push(format!("{}: {}", name, v)); }
                };
            
                add("STR", stats.total_str);
                add("INT", stats.total_int);
                add("END", stats.total_end);
                add("WIS", stats.total_wis);
                add("DEX", stats.total_dex);
                add("LUK", stats.total_lck);
                
                add("Attack Power", stats.attack_power);
                add("Spell Power", stats.spell_power);
                add("All Out", stats.damage_boost_all);
                add("Physical Out", stats.physical_boost);
                add("Magic Out", stats.magic_boost);
                add("Dot Out", stats.dot_boost);
                add("Heal Out", stats.heal_boost);
                add("All In", stats.damage_intake);
                add("Physical In", stats.physical_intake);
                add("Magic In", stats.magic_intake);
                add("Dot In", stats.dot_intake);
                add("Heal In", stats.healing_intake);
                
                add("Mana Consumption", stats.mana_consumption);
                add("Crit Chance", stats.crit_rate);
                add("Dodge", stats.dodge_chance);
                add("Haste", stats.haste);
                add("Dash", stats.dash);
                add("Health Boost", stats.health_boost);
                add("Mana Boost", stats.mana_boost);
                add("Hit Chance", stats.hit_chance);
                add("Crit Mod", stats.crit_mod);
            
                if !changed.is_empty() {
                    self.push_log(format!("  -> [Stats] Updated: {}", changed.join(", ")));
                }
            }
            
            ScribeEvent::Death { victim, killer } => {
                self.push_log(format!("  -> [DEATH] {} was killed by {}", victim, killer));
            }

            ScribeEvent::ZoneChange { room } => {
                self.push_log(format!("\n>>> ROOM SHIFT: {} <<<\n", room));
                self.last_vitals.clear();
            }

            ScribeEvent::StateChange { username, state, level, entity_id } => {
                let status = match state {
                    2 => "Entered Combat",
                    1 => "Alive",
                    0 => "Fainted",
                    _ => "Unknown",
                };
                self.push_log(format!("[State] {} (Lvl: {}, ID: {}) is now: {}", username, level, entity_id, status));
            }

            ScribeEvent::CombatTick { stats, auras } => {
                for stat in stats {
                    let mut updates = Vec::new();
                    let (prev_hp, prev_mp, prev_sh) = self.last_vitals
                        .get(&stat.target)
                        .cloned()
                        .unwrap_or((stat.hp.unwrap_or(0), stat.mp.unwrap_or(0), stat.shield.unwrap_or(0)));
                    
                    if let Some(new_hp) = stat.hp {
                        let diff = new_hp - prev_hp;
                        if diff != 0 {
                            let label = if diff > 0 { "Heal" } else { "Dmg" };
                            updates.push(format!("{}: {} ({})", label, diff.abs(), new_hp));
                        }
                    }
                    if let Some(new_mp) = stat.mp {
                        let diff = new_mp - prev_mp;
                        if diff != 0 {
                            let label = if diff > 0 { "MP Gain" } else { "MP Cost" };
                            updates.push(format!("{}: {} ({})", label, diff.abs(), new_mp));
                        }
                    }
                    if let Some(new_sh) = stat.shield {
                        let diff = new_sh - prev_sh;
                        if diff != 0 {
                            let label = if diff > 0 { "SG Gain" } else { "SG Loss" };
                            updates.push(format!("{}: {} ({})", label, diff.abs(), new_sh));
                        }
                    }

                    self.last_vitals.insert(
                        stat.target.clone(), 
                        (stat.hp.unwrap_or(prev_hp), stat.mp.unwrap_or(prev_mp), stat.shield.unwrap_or(prev_sh))
                    );
                            
                    if !updates.is_empty() {
                        self.push_log(format!("  -> [Vitals] {} -> {}", stat.target, updates.join(" | ")));
                    }
                }

                for aura in auras {
                    let meta_str = Self::format_aura_metadata(&aura);
                    match aura.action.as_str() {
                        "Applied" => self.push_log(format!("  -> [+] [Aura] {} applied to {} (by {}){}", aura.aura_name, aura.target, aura.caster, meta_str)),
                        "Refreshed" => self.push_log(format!("  -> [*] [Aura] {} refreshed on {}{}", aura.aura_name, aura.target, meta_str)),
                        "Faded" => self.push_log(format!("  -> [-] [Aura] {} expired from {}{}", aura.aura_name, aura.target, meta_str)),
                        _ => self.push_log(format!("  -> [?] [Aura] {} {} on {}{}", aura.aura_name, aura.action, aura.target, meta_str)),
                    }
                }
            }
            ScribeEvent::Unknown { data } => {
                self.push_log(format!("  -> [?] Unknown event: {}", data));
            }
        }
    }
}