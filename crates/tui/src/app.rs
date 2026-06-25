use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::Write;
use scribe_parser::events::{ScribeEvent, AuraTimelineDelta};

const MAX_HISTORY: usize = 1000; // Reduced for UI memory safety

#[derive(Debug, Clone)]
pub struct EntityState {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32, // Note: You may need to populate max stats from StateChange events
    pub mp: i32,
    pub shield: i32,
    pub auras: Vec<String>,
    pub last_action: String,
}

impl EntityState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            hp: 0,
            max_hp: 0, 
            mp: 0,
            shield: 0,
            auras: Vec::new(),
            last_action: "Spawned".to_string(),
        }
    }
}

pub struct App {
    // The live state of the room for your TUI Table/Spreadsheet
    pub entities: HashMap<String, EntityState>,
    
    // A scrolling log for the TUI (Room shifts, Death Recaps)
    pub system_log: VecDeque<String>,
    
    // Keeps a history of raw events specifically for the Death Recap calculation
    recent_events: VecDeque<ScribeEvent>, 
    
    // Optional: Keeps writing to your log file in the background
    log_file: Option<File>,
    
    pub should_quit: bool,
}

impl App {
    pub fn new(log_path: Option<&str>) -> Self {
        let log_file = log_path.map(|path| {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("Failed to open log file")
        });

        Self {
            entities: HashMap::new(),
            system_log: VecDeque::with_capacity(MAX_HISTORY),
            recent_events: VecDeque::with_capacity(MAX_HISTORY),
            log_file,
            should_quit: false,
        }
    }

    /// Appends to the UI's scrolling text box AND the background text file
    fn push_log(&mut self, message: String) {
        if let Some(file) = &mut self.log_file {
            let _ = writeln!(file, "{}", message);
        }
        
        if self.system_log.len() >= MAX_HISTORY {
            self.system_log.pop_front();
        }
        self.system_log.push_back(message);
    }

    /// Helper to grab an entity or create it if we haven't seen it yet
    fn get_or_create_entity(&mut self, target: &str) -> &mut EntityState {
        self.entities.entry(target.to_string())
            .or_insert_with(|| EntityState::new(target.to_string()))
    }

    pub fn handle_event(&mut self, event: ScribeEvent) {
        // 1. Process the visual state updates
        match &event {
            ScribeEvent::Death { victim, killer } => {
                self.print_death_recap(victim, killer);
                if let Some(ent) = self.entities.get_mut(victim) {
                    ent.last_action = format!("💀 Killed by {}", killer);
                    ent.hp = 0;
                }
            }
            ScribeEvent::ZoneChange { room } => {
                self.push_log(format!(">>> ROOM SHIFT: {} <<<", room));
                // Wipe the spreadsheet! We are in a new room.
                self.entities.clear(); 
            }
            ScribeEvent::StateChange { username, state, level, entity_id } => {
                let status = match state {
                    2 => "Entered Combat",
                    1 => "Alive",
                    0 => "Fainted",
                    _ => "Unknown",
                };
                
                let ent = self.get_or_create_entity(username);
                ent.last_action = status.to_string();
                
                self.push_log(format!(
                    "[State] {} (Lvl: {}, ID: {}) is now: {}", 
                    username, level, entity_id, status
                ));
            }
            ScribeEvent::CombatTick { stats, auras } => {
                // Handle Vitals
                for stat in stats {
                    let ent = self.get_or_create_entity(&stat.target);
                    let mut action_labels = Vec::new();

                    if let Some(new_hp) = stat.hp {
                        let diff = new_hp - ent.hp;
                        if diff != 0 {
                            let label = if diff > 0 { "Heal" } else { "Dmg" };
                            action_labels.push(format!("{}: {}", label, diff.abs()));
                        }
                        ent.hp = new_hp;
                    }
                    
                    if let Some(new_mp) = stat.mp {
                        ent.mp = new_mp;
                    }
                    
                    if let Some(new_sh) = stat.shield {
                        ent.shield = new_sh;
                    }

                    if !action_labels.is_empty() {
                        ent.last_action = action_labels.join(" | ");
                    }
                }

                // Handle Auras
                for aura in auras {
                    let ent = self.get_or_create_entity(&aura.target);
                    
                    match aura.action.as_str() {
                        "Applied" | "Refreshed" => {
                            // Add to list if it's not already there
                            if !ent.auras.contains(&aura.aura_name) {
                                ent.auras.push(aura.aura_name.clone());
                            }
                            ent.last_action = format!("Gained: {}", aura.aura_name);
                        }
                        "Faded" => {
                            // Remove from list
                            ent.auras.retain(|a| a != &aura.aura_name);
                            ent.last_action = format!("Lost: {}", aura.aura_name);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        // 2. Store the event for the Death Recap
        if self.recent_events.len() >= MAX_HISTORY {
            self.recent_events.pop_front();
        }
        self.recent_events.push_back(event);
    }

    fn print_death_recap(&mut self, victim: &str, killer: &str) {
        self.push_log("==================================================".to_string());
        self.push_log("                PLAYER DEATH DETECTED               ".to_string());
        self.push_log(format!("  Victim: {} | Fatality Delivered By: {}", victim.to_uppercase(), killer));
        self.push_log("==================================================".to_string());
        self.push_log("RECENT TIMELINE:".to_string());

        let events = self.recent_events.clone().into_iter().collect::<Vec<_>>();
        for past_event in &events {
            match past_event {
                ScribeEvent::CombatTick { stats, auras } => {
                    for aura in auras {
                        if aura.target.to_lowercase() == victim.to_lowercase() {
                            let val_str = aura.value.map(|v| format!(" (Value: {})", v)).unwrap_or_default();
                            self.push_log(format!("   Aura [{}] {} by {}{}",
                                aura.aura_name, aura.action.to_uppercase(), aura.caster, val_str
                            ));
                        }
                    }
                    for stat in stats {
                        if stat.target.to_lowercase() == victim.to_lowercase() {
                            if let Some(hp) = stat.hp {
                                self.push_log(format!("   Vitals | HP updated to: {}", hp));
                            }
                            if let Some(shield) = stat.shield {
                                self.push_log(format!("   Shield | Safeguard Capacity: {}", shield));
                            }
                        }
                    }
                }
                ScribeEvent::StateChange { username, state, level, entity_id} if username.to_lowercase() == victim.to_lowercase() => {
                    self.push_log(format!("[State] {} (Lvl: {}, ID: {}) is now: {}", username, level, entity_id, state));
                }
                _ => {}
            }
        }

        self.push_log("--------------------------------------------------".to_string());
        self.push_log(format!("  FATAL DAMAGE TICK: {} killed {}", killer, victim));
        self.push_log("==================================================".to_string());
    }
}