use std::collections::VecDeque;
use scribe_parser::events::{ScribeEvent, AuraTimelineDelta, StatTimelineDelta};
use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::HashMap;

const MAX_HISTORY: usize = 10000;

pub struct ScribeOutput {
    history: VecDeque<ScribeEvent>,
    log_file: Arc<Mutex<std::fs::File>>,
    last_vitals: HashMap<String, (i32, i32, i32)>,
}

impl ScribeOutput {
    pub fn new(log_path: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Failed to open log file");

        Self {
            history: VecDeque::with_capacity(MAX_HISTORY),
            log_file: Arc::new(Mutex::new(file)),
            last_vitals: HashMap::new(),
        }
    }

    fn log(&self, message: &str) {
        println!("{}", message);
        if let Ok(mut file) = self.log_file.lock() {
            let _ = writeln!(file, "{}", message);
        }
    }

        

    pub fn handle_event(&mut self, event: ScribeEvent) {
        self.stream_live_update(&event);
        if self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(event);
    }

    fn format_aura_metadata(aura: &AuraTimelineDelta) -> String {
        let mut meta_segments = Vec::new();

        if let Some(dur) = aura.duration {
            meta_segments.push(format!("Duration: {}s", dur));
        }
        if let Some(val) = aura.value {
            meta_segments.push(format!("Value: {}", val));
        }
        if let Some(ref aura_type) = aura.aura_type {
            meta_segments.push(format!("Type: {}", aura_type));
        }
        if let Some(is_new) = aura.is_new {
            meta_segments.push(format!("New: {}", is_new));
        }

        if meta_segments.is_empty() {
            String::new()
        } else {
            format!(" [{}]", meta_segments.join(" | "))
        }
    }

    fn stream_live_update(&mut self, event: &ScribeEvent) {
        match event {
            ScribeEvent::Death { victim, killer } => {
                self.print_death_recap(victim, killer);
            }
            ScribeEvent::ZoneChange { room } => {
                self.log(&format!("\n>>> ROOM SHIFT: {} <<<\n", room));
            }
            ScribeEvent::StateChange { username, state, level, entity_id } => {
                let status = match state {
                    2 => "Entered Combat",
                    1 => "Alive",
                    0 => "Fainted",
                    _ => "Unknown",
                };
                self.log(&format!(
                    "[State] {} (Lvl: {}, ID: {}) is now: {}", 
                    username, level, entity_id, status
                ));
            },

            ScribeEvent::CombatTick { stats, auras } => {
                for stat in stats {
                    let mut updates = Vec::new();
                    let (prev_hp, prev_mp, prev_sh) = self.last_vitals.get(&stat.target).cloned().unwrap_or((stat.hp.unwrap_or(0), stat.mp.unwrap_or(0), stat.shield.unwrap_or(0) ));
                    
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
                        self.log(&format!("  -> [Vitals] {} -> {}", stat.target, updates.join(" | ")));
                    }
                }


                for aura in auras {
                    let meta_str = Self::format_aura_metadata(aura);
                    match aura.action.as_str() {
                        "Applied" => {
                            self.log(&format!("  -> [+] [Aura] {} applied to {} (by {}){}", aura.aura_name, aura.target, aura.caster, meta_str));
                        }
                        "Refreshed" => {
                            self.log(&format!("  -> [*] [Aura] {} refreshed on {}{}", aura.aura_name, aura.target, meta_str));
                        }
                        "Faded" => {
                            self.log(&format!("  -> [-] [Aura] {} expired from {}{}", aura.aura_name, aura.target, meta_str));
                        }
                        _ => {
                            self.log(&format!("  -> [?] [Aura] {} {} on {}{}", aura.aura_name, aura.action, aura.target, meta_str));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn print_death_recap(&self, victim: &str, killer: &str) {
        self.log("\n==================================================");
        self.log("                PLAYER DEATH DETECTED               ");
        self.log(&format!("  Victim: {} | Fatality Delivered By: {}", victim.to_uppercase(), killer));
        self.log("==================================================");
        self.log("RECENT TIMELINE (Oldest to Newest):");
        self.log("--------------------------------------------------");

        for past_event in &self.history {
            match past_event {
                ScribeEvent::CombatTick { stats, auras } => {
                    for aura in auras {
                        if aura.target.to_lowercase() == victim.to_lowercase() {
                            let val_str = aura.value.map(|v| format!(" (Value: {})", v)).unwrap_or_default();
                            self.log(
                                &format!("   Aura [{}] {} by {}{}",
                                    aura.aura_name,
                                    aura.action.to_uppercase(),
                                    aura.caster,
                                    val_str
                                )
                            );
                        }
                    }

                    for stat in stats {
                        if stat.target.to_lowercase() == victim.to_lowercase() {
                            if let Some(hp) = stat.hp {
                                self.log(&format!("   Vitals | HP updated to: {}", hp));
                            }
                            if let Some(shield) = stat.shield {
                                self.log(&format!("   Shield | Safeguard Capacity: {}", shield));
                            }
                        }
                    }
                }
                ScribeEvent::StateChange { username, state, level, entity_id} => {
                    if username.to_lowercase() == victim.to_lowercase() {
                        self.log(&format!(
                            "[State] {} (Lvl: {}, ID: {}) is now: {}", 
                            username, level, entity_id, state
                        ));
                    }
                }
                _ => {}
            }
        }

        self.log("--------------------------------------------------");
        self.log(&format!("  FATAL DAMAGE TICK: {} killed {}", killer, victim));
        self.log("==================================================\n");
    }
}