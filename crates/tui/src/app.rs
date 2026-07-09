use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{Instant, Duration};
use scribe_parser::events::{ScribeEvent, AuraTimelineDelta};
use serde_json::{Map, Value};
use scribe_core::mechanics::{Strategy, Action, BossRegistry, Trigger};


pub type DamageInstance = (i32, Instant);
const INSTANCE_MAX: usize = 20;

#[derive(Debug, Clone)]
pub struct DPSTracker {
    pub damage_instances: Vec<DamageInstance>,
    pub instance_time: Duration,
    pub group_total_damage: i32,
    pub session_start: Instant,
    pub last_activity: Instant,

    pub solo_total_damage: i32,
    pub solo_damage_instances: Vec<DamageInstance>,
}


impl DPSTracker {
    pub fn recent_group_dps(&self) -> f64 {
        if self.damage_instances.len() >= (INSTANCE_MAX / 2) {
            let first = &self.damage_instances[0];
            let last = &self.damage_instances[self.damage_instances.len() - 1];
            let hp_diff = (first.0 - last.0) as f64;
            let time_diff = last.1.duration_since(first.1).as_secs_f64();

            if time_diff > 0.0 && hp_diff > 0.0 {
                hp_diff / time_diff
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}


impl Default for DPSTracker {
    fn default() -> Self {
        Self {
            instance_time: Duration::default(),
            damage_instances: Vec::new(),
            group_total_damage: 0,
            session_start: Instant::now(),
            last_activity: Instant::now(),
            solo_total_damage: 0,
            solo_damage_instances: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpeakerMechanicsState {
    pub active_boss: Vec<Strategy>,
    pub strategy_index: usize,
    pub current_phase: usize,
    pub selected_role: String,
    pub active_alert: Option<(Action, Instant)>,
    pub last_prompt: String,
    pub last_prompt_time: Instant,
    pub active_auras: HashMap<String, Instant>,
}

impl Default for SpeakerMechanicsState {
    fn default() -> Self {
        Self {
            active_boss: Vec::new(),
            strategy_index: 0,
            current_phase: 0,
            selected_role: "LR".to_string(),
            active_alert: None,
            last_prompt: String::new(),
            last_prompt_time: Instant::now() - Duration::from_secs(60),
            active_auras: HashMap::new(),

        }
    }
}
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
pub struct MonsterCombatState {
    pub base_hp: i32,
    pub damage_dealt: i32,
    pub total_damage_dealt: i32,
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


#[derive(Debug, Clone)]
pub struct DropMetric {
    pub id: u32,
    pub name: String,
    pub total_quantity: u32,
    pub drop_count: u32,
    pub kills_at_last_drop: u32,
}

#[derive(Debug, Clone)]
pub struct CombatMetrics {
    pub session_start: Instant,
    pub last_activity: Instant,
    pub total_kills: u32,
    pub total_gold: i32,
    pub total_exp: i32,
    pub dps: DPSTracker,
    pub drops: HashMap<String, DropMetric>,
}

impl Default for CombatMetrics {
    fn default() -> Self {
        Self {
            session_start: Instant::now(),
            last_activity: Instant::now(),
            total_kills: 0,
            total_gold: 0,
            total_exp: 0,
            drops: HashMap::new(),
            dps: DPSTracker::default(),
        }
    }
}


impl CombatMetrics {
    fn reset_farming(&mut self) {
        self.total_kills = 0;
        self.total_gold = 0;
        self.total_exp = 0;
        self.drops.clear();
        self.session_start = Instant::now();
        self.last_activity = Instant::now();
    }
}


pub struct App {
    pub active_tab: usize,
    pub tabs: Vec<&'static str>,
    pub scroll_output_y: u16,
    pub scroll_output_x: u16,
    pub scroll_class_x: u16,
    pub scroll_class_y: u16,
    pub scroll_metrics_y: u16,

    pub snap: bool,
    pub active_monsters: HashMap<String, MonsterCombatState>,

    pub entities: HashMap<String, EntityState>,
    pub system_log: VecDeque<String>,
    pub last_vitals: HashMap<String, (i32, i32, i32)>,
    recent_events: VecDeque<ScribeEvent>,
    log_file: Option<File>,
    pub should_quit: bool,
    pub users_in_room: Vec<String>,

    pub class_info: ClassInfo,
    pub selected_skill_index: usize,
    pub combat_metrics: CombatMetrics,
    pub item_cache: std::collections::HashMap<u32, String>,
    pub username: String,

    pub mechanics: SpeakerMechanicsState,

}

impl App {
    pub fn new(log_path: Option<&str>, username: String) -> Self {
        let log_file = log_path.map(|path| {
            OpenOptions::new().create(true).append(true).open(path).expect("Failed to open log file")
        });

        Self {
            active_tab: 0,
            tabs: vec!["Output Log", "Class Data", "Combat Metrics", "Ultra Mechanics"],
            scroll_output_y: 0,
            scroll_output_x: 0,
            scroll_class_x: 0,
            scroll_class_y: 0,
            scroll_metrics_y: 0,
            entities: HashMap::new(),
            system_log: VecDeque::with_capacity(1000),
            last_vitals: HashMap::new(),
            recent_events: VecDeque::with_capacity(1000),
            log_file,
            should_quit: false,
            users_in_room: Vec::new(),
            username,
            snap: false,
            active_monsters: HashMap::new(),
            class_info: ClassInfo::default(),
            selected_skill_index: 0,
            combat_metrics: CombatMetrics::default(),
            item_cache: std::collections::HashMap::new(),
            mechanics: SpeakerMechanicsState::default()
        }
    }

    pub fn snap_to_bottom(&mut self) {
        self.snap = true;
    }

    fn update_dps(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.combat_metrics.dps.last_activity) > Duration::from_secs(60) {
            self.combat_metrics.dps = DPSTracker::default();
        } else {
            self.combat_metrics.dps.last_activity = now;
        }

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

            ScribeEvent::PlayerDamage { caster, target, damage } => {
                if caster == self.username {
                    let state = self.active_monsters.entry(target.clone()).or_default();
                    state.damage_dealt += damage;
                    self.update_dps();
                    self.combat_metrics.dps.solo_total_damage += damage;

                    if self.combat_metrics.dps.solo_damage_instances.len() >= INSTANCE_MAX {
                        self.combat_metrics.dps.solo_damage_instances.remove(0);
                    }
                    self.combat_metrics.dps.solo_damage_instances.push((damage, Instant::now()));
                    self.push_log(format!("  -> [Combat] Target: {} | Your DMG: {}", target, damage));

                }
                self.push_log(format!("  -> [Combat] Caster: {} | Target: {} | DMG: {}", caster, target, damage));

            }
            ScribeEvent::EnemyDamage { caster: _, target, damage } => {

                let state = self.active_monsters.entry(target.clone()).or_default();
                state.damage_dealt += damage;
                self.update_dps();
                self.combat_metrics.dps.solo_total_damage += damage;

                if self.combat_metrics.dps.solo_damage_instances.len() >= 10 {
                    self.combat_metrics.dps.solo_damage_instances.remove(0);
                }
                self.combat_metrics.dps.solo_damage_instances.push((damage, Instant::now()));

                self.push_log(format!("  -> [Combat] Target: {} | Your DMG: {}", target, damage));
            }

            ScribeEvent::MonsterReset { target, base_hp } => {
                self.mechanics.current_phase = 0;
                self.mechanics.active_alert = None;
                self.mechanics.last_prompt.clear();

                self.last_vitals.remove(&target);

                if let Some(state) = self.active_monsters.get(&target) {
                    let contribution = if state.total_damage_dealt > 0 {
                        (state.damage_dealt as f64 / state.total_damage_dealt as f64) * 100.0

                    } else { 0.0 };

                    let mon_hp_contribution = if base_hp > 0 {
                        (state.damage_dealt as f64 / base_hp as f64) * 100.0
                    } else { 0.0 };

                    self.push_log(format!(
                        "  -> [Combat] Target: {} | Base HP: {} | Your Dmg: {} | Pct: {:.1}% | HP Pct: {:.1}%",
                        target, state.base_hp, state.damage_dealt, contribution, mon_hp_contribution
                    ));
                }


                self.active_monsters.insert(target.clone(), MonsterCombatState {
                    base_hp,
                    damage_dealt: 0,
                    total_damage_dealt: 0,
                });
            }

            ScribeEvent::InventoryLoaded { items } => {
                self.item_cache.extend(items);
                self.push_log(format!("  -> [System] Cached {} items from inventory", self.item_cache.len()));
            }

            ScribeEvent::ItemAdded { item_id, quantity, quantity_now } => {
                self.combat_metrics.reset_farming();

                let current_kills = self.combat_metrics.total_kills;

                let item_name = self.item_cache.get(&item_id).cloned().unwrap_or_else(|| format!("Item #{}", item_id));

                let entry = self.combat_metrics.drops.entry(item_name.clone()).or_insert(DropMetric {
                    id: item_id,
                    name: item_name.clone(),
                    total_quantity: 0,
                    drop_count: 0,
                    kills_at_last_drop: current_kills,
                });

                entry.total_quantity += quantity;
                entry.drop_count += 1;
                entry.kills_at_last_drop = current_kills;

                self.push_log(format!(
                    "  -> [Bag] Added {}x {} (Total in Bag: {})",
                    quantity, item_name, quantity_now
                ));
            }

            ScribeEvent::ItemDropped { item_id, item_name, quantity } => {
                let current_kills = self.combat_metrics.total_kills;

                self.item_cache.insert(item_id, item_name.clone());

                let entry = self.combat_metrics.drops.entry(item_name.clone()).or_insert(DropMetric {
                    id: item_id,
                    name: item_name.clone(),
                    total_quantity: 0,
                    drop_count: 0,
                    kills_at_last_drop: current_kills,
                });

                entry.total_quantity += quantity;
                entry.drop_count += 1;
                entry.kills_at_last_drop = current_kills;

                self.push_log(format!("  -> [Drop] {}x {} (ID: {})", quantity, item_name, item_id));
                self.combat_metrics.reset_farming();
            }

            ScribeEvent::RoomPlayersUpdate { players } => {
                self.users_in_room = players;
            }

            ScribeEvent::GoldExpGained { monster_name, gold, exp, bonus_gold } => {
                let bonus = if bonus_gold > 0 { format!(" (+{} bonus)", bonus_gold) } else { "".to_string() };
                self.push_log(format!("  -> [Loot] Killed {}: {} Gold{} | {} Exp", monster_name, gold, bonus, exp));
                self.combat_metrics.total_kills += 1;
                self.combat_metrics.total_gold += gold + bonus_gold;
                self.combat_metrics.total_exp += exp;
                self.combat_metrics.reset_farming();
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

            ScribeEvent::ClassUpdated { uid: _, class_name, category, desc, mrm } => {
                if !desc.is_empty() {
                    self.class_info.name = class_name.clone();
                    self.class_info.category = category.clone();
                    self.class_info.desc = desc.clone();
                    self.class_info.mrm = mrm.clone();
                    self.selected_skill_index = 0;
                    self.class_info.active_skills = Vec::new();
                    self.class_info.passive_skills = Vec::new();
                }

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
                        if let Some(existing_skill) = self.class_info.passive_skills.get_mut(i) {
                            existing_skill.extend(skill.clone());
                        } else {
                            self.class_info.passive_skills.push(skill.clone());
                        }
                    }
                }
                self.selected_skill_index = 0;

                self.push_log(format!("  -> Loaded {} Active & {} Passive Skills",
                    self.class_info.active_skills.len(),
                    self.class_info.passive_skills.len()
                ));

            }

            ScribeEvent::UserDataInitialized { username, uid, class_name, access_level: _ } => {
                let class_str = class_name;
                self.push_log(format!("  -> User data initialized: {} (UID: {}, Class: {})", username, uid, class_str));
            }

            ScribeEvent::BossAction { caster, target, message, action_type } => {
                let msg_part = if message.is_empty() { String::new() } else { format!(" | msg: {}", message) };
                self.push_log(format!("  -> [Boss] {} >> {} : ({}){}", caster, target, action_type, msg_part));

                let now = std::time::Instant::now();

                let is_duplicate = self.mechanics.last_prompt == message &&
                    now.duration_since(self.mechanics.last_prompt_time) < std::time::Duration::from_millis(500);

                if !is_duplicate {
                    let strategies = self.mechanics.active_boss.clone();

                    if let Some(strategy) = strategies.get(self.mechanics.strategy_index) {

                        let msg_lower = message.to_lowercase();
                        let mut reactive_triggered = false;


                        if let Some(reactives) = &strategy.reactives {
                            for reactive in reactives {
                                if let scribe_core::mechanics::Trigger::Message(msg) = &reactive.trigger {
                                    if msg_lower.contains(&msg.to_lowercase()) {
                                        reactive_triggered = true;
                                        if reactive.role == self.mechanics.selected_role || reactive.role == "all" {
                                            self.mechanics.active_alert = Some((reactive.action.clone(), now));
                                            self.push_log(format!("  -> [Reactive] YOUR TURN: {:?}", reactive.action));
                                        } else {
                                            self.mechanics.active_alert = None;
                                            self.push_log(format!("  -> [Reactive] {}'s turn to {:?}", reactive.role, reactive.action));
                                        }
                                        break; // Stop checking other reactives
                                    }
                                }
                            }
                        }


                        if !reactive_triggered {
                            let mut found_match = false;
                            let mut matched_step = None;
                            let mut new_phase = self.mechanics.current_phase;

                            for offset in 0..=2 {
                                let check_phase = self.mechanics.current_phase + offset;

                                let opening_len = strategy.opening.as_ref().map_or(0, |o| o.len());

                                let step = if check_phase < opening_len {
                                    strategy.opening.as_ref().unwrap().get(check_phase)
                                } else {
                                    match &strategy.cycle {
                                        Some(cycle) if !cycle.is_empty() => {
                                            let cycle_idx = (check_phase - opening_len) % cycle.len();
                                            cycle.get(cycle_idx)
                                        }
                                        _ => None,
                                    }
                                };

                                if let Some(s) = step {
                                    if let Trigger::Message(ref prompt) = s.trigger {
                                        if msg_lower.contains(&prompt.to_lowercase()) {
                                            found_match = true;
                                            matched_step = Some(s.clone());
                                            new_phase = check_phase + 1;
                                            break;
                                        }
                                    }
                                }
                            }

                            if found_match {
                                if let Some(step) = matched_step {
                                    self.mechanics.last_prompt = message.clone();
                                    self.mechanics.last_prompt_time = now;

                                    self.mechanics.current_phase = new_phase;

                                    if step.role == self.mechanics.selected_role || step.role == "all" {
                                        self.mechanics.active_alert = Some((step.action.clone(), now));
                                        self.push_log(format!("  -> [Speaker] Phase {}! YOUR TURN: {:?}", self.mechanics.current_phase, step.action));
                                    } else {
                                        self.mechanics.active_alert = None;
                                        self.push_log(format!("  -> [Speaker] Phase {} ({}'s turn to {:?})", self.mechanics.current_phase, step.role, step.action));
                                    }
                                }
                            } else if msg_lower.contains("truth") || msg_lower.contains("listen") || msg_lower.contains("equal") {
                                self.push_log(format!("  -> [Mechanic Desync] Tracker at {}, Boss said: {}", self.mechanics.current_phase, msg_lower));
                            }
                        }
                    }
                }
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

                self.mechanics.current_phase = 0;
                self.mechanics.active_alert = None;
                self.mechanics.last_prompt.clear();
                self.mechanics.active_boss.clear();

                match std::fs::read_to_string("./mechanics.json") {
                    Ok(json_str) => {

                        match serde_json::from_str::<BossRegistry>(&json_str) {
                            Ok(registry) => {
                                let room_lower = room.to_lowercase();


                                if let Some((boss_key, strategies)) = registry.iter().find(|(key, _)| room_lower.contains(*key)) {
                                    self.mechanics.active_boss = strategies.clone();
                                    self.mechanics.strategy_index = 0;
                                    self.push_log(format!("  -> [Mechanics] Loaded strategies for: {}", boss_key));
                                }
                            }
                            Err(e) => {
                                self.push_log(format!("  -> [Mechanics Error] Failed to parse mechanics.json: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        self.push_log(format!("  -> [Mechanics Error] Could not read mechanics.json: {}", e));
                    }
                }

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
                            if let Some(state) = self.active_monsters.get_mut(&stat.target) {

                                if diff < 0 {
                                    let dmg = diff.abs();
                                    state.total_damage_dealt += dmg;

                                    self.update_dps();
                                    if self.combat_metrics.dps.damage_instances.len() >= INSTANCE_MAX {
                                        self.combat_metrics.dps.damage_instances.remove(0);
                                    }
                                    self.combat_metrics.dps.damage_instances.push((new_hp, Instant::now()));
                                    self.combat_metrics.dps.group_total_damage += dmg;
                                    self.combat_metrics.dps.instance_time = Instant::now() - self.combat_metrics.dps.damage_instances[0].1;
                                }
                            }
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
                            if let Some(state) = self.active_monsters.get_mut(&stat.target) {
                                if diff < 0 {
                                    let dmg = diff.abs();
                                    state.total_damage_dealt += dmg;

                                    self.update_dps();
                                    if self.combat_metrics.dps.damage_instances.len() >= INSTANCE_MAX {
                                        self.combat_metrics.dps.damage_instances.remove(0);
                                    }
                                    self.combat_metrics.dps.damage_instances.push((new_sh, Instant::now()));
                                    self.combat_metrics.dps.group_total_damage += dmg;
                                    self.combat_metrics.dps.instance_time = Instant::now() - self.combat_metrics.dps.damage_instances[0].1;
                                }

                            }
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
                        "Applied" | "Refreshed" => {
                            if let Some(d) = aura.duration {
                                let dur_secs = format!("{}", d).parse::<f64>().unwrap_or(0.0);
                                self.mechanics.active_auras.insert(
                                    aura.aura_name.clone(),
                                    Instant::now() + Duration::from_secs_f64(dur_secs)
                                );
                            }
                            let symbol = if aura.action == "Applied" { "+" } else { "*" };
                            self.push_log(format!("  -> [{}] [Aura] {} {} on {}{}", symbol, aura.aura_name, aura.action, aura.target, meta_str));

                            let strategies = self.mechanics.active_boss.clone();
                            if let Some(strategy) = strategies.get(self.mechanics.strategy_index) {
                                let check_phase = self.mechanics.current_phase;
                                let opening_len = strategy.opening.as_ref().map_or(0, |o| o.len());
                                let current_step = if check_phase < opening_len {
                                    strategy.opening.as_ref().unwrap().get(check_phase)
                                } else {
                                    match &strategy.cycle {
                                        Some(cycle) if !cycle.is_empty() => cycle.get((check_phase - opening_len) % cycle.len()),
                                        _ => None,
                                    }
                                };

                                if let Some(step) = current_step {
                                    if let scribe_core::mechanics::Trigger::Aura { name, .. } = &step.trigger {
                                        if name.to_lowercase() == aura.aura_name.to_lowercase() {
                                            self.mechanics.current_phase += 1; // Advance!
                                            if let Some((active_act, _)) = &self.mechanics.active_alert {
                                                if *active_act == step.action { self.mechanics.active_alert = None; }
                                            }
                                            self.push_log(format!("  -> [Mechanics] Phase {} cleared ({} refreshed)", check_phase, aura.aura_name));
                                        }
                                    }
                                }
                            }
                        },
                        "Faded" => {
                            self.mechanics.active_auras.remove(&aura.aura_name);
                            self.push_log(format!("  -> [-] [Aura] {} expired from {}{}", aura.aura_name, aura.target, meta_str));
                        },
                        _ => self.push_log(format!("  -> [?] [Aura] {} {} on {}{}", aura.aura_name, aura.action, aura.target, meta_str)),
                    }
                }
            }
            ScribeEvent::Unknown { data } => {
                self.push_log(format!("  -> [?] Unknown event: {}", data));
            }
        }
    }


    pub fn tick(&mut self) {
        let now = Instant::now();
        let strategies = self.mechanics.active_boss.clone();
        let recent_dps = self.combat_metrics.dps.recent_group_dps();

        if let Some(strategy) = strategies.get(self.mechanics.strategy_index) {
            let mut pending_alert: Option<Action> = None;

            if let Some(reactives) = &strategy.reactives {
                for reactive in reactives {
                    if reactive.role == self.mechanics.selected_role || reactive.role == "all" {
                        let mut triggered = false;

                        match &reactive.trigger {
                            scribe_core::mechanics::Trigger::Aura { name, timer } => {
                                if let Some(expiration) = self.mechanics.active_auras.get(name) {
                                    let remaining = expiration.saturating_duration_since(now);
                                    let buffer = Duration::from_millis(*timer as u64);

                                    if remaining > Duration::ZERO && remaining <= buffer {
                                        triggered = true;
                                    }
                                }
                            },
                            scribe_core::mechanics::Trigger::HP { value, timer } => {
                                let current_hp = self.last_vitals.values().map(|(hp, _, _)| *hp).max().unwrap_or(0);
                                let target_hp = *value as i32;

                                if current_hp > target_hp {
                                    let hp_remaining = (current_hp - target_hp) as f64;
                                    let buffer_secs = (*timer as f64) / 1000.0;
                                    let time_to_threshold = if recent_dps > 0.0 { hp_remaining / recent_dps } else { f64::MAX };
                                    
                                    if time_to_threshold <= buffer_secs || hp_remaining <= 300_000.0 {
                                        triggered = true;
                                    } else if let Some((active_act, _)) = &self.mechanics.active_alert {
                                        if *active_act == reactive.action && time_to_threshold <= (buffer_secs * 2.0) {
                                            triggered = true;
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }

                        if triggered {
                            pending_alert = Some(reactive.action.clone());
                            break;
                        }
                    }
                }
            }

            let check_phase = self.mechanics.current_phase;
            let opening_len = strategy.opening.as_ref().map_or(0, |o| o.len());

            let current_step = if check_phase < opening_len {
                strategy.opening.as_ref().unwrap().get(check_phase)
            } else {
                match &strategy.cycle {
                    Some(cycle) if !cycle.is_empty() => cycle.get((check_phase - opening_len) % cycle.len()),
                    _ => None,
                }
            };

            if let Some(step) = current_step {
                let is_my_turn = step.role == self.mechanics.selected_role || step.role == "all";

                match &step.trigger {
                    scribe_core::mechanics::Trigger::Aura { name, timer } => {
                        if let Some(expiration) = self.mechanics.active_auras.get(name) {
                            let remaining = expiration.saturating_duration_since(now);
                            let buffer = Duration::from_millis(*timer as u64);

                            if remaining > Duration::ZERO && remaining <= buffer && is_my_turn {
                                if pending_alert.is_none() {
                                    pending_alert = Some(step.action.clone());
                                }
                            }
                        }
                    },
                    scribe_core::mechanics::Trigger::HP { value, timer } => {
                        let current_hp = self.last_vitals.values().map(|(hp, _, _)| *hp).max().unwrap_or(0);
                        let target_hp = *value as i32;

                        if current_hp > target_hp {
                            let time_to_threshold = if recent_dps > 0.0 { ((current_hp - target_hp) as f64) / recent_dps } else { f64::MAX };

                            if time_to_threshold <= (*timer as f64) / 1000.0 && is_my_turn {
                                if pending_alert.is_none() {
                                    pending_alert = Some(step.action.clone());
                                }
                            }
                        } else if current_hp > 0 {
                            self.mechanics.current_phase += 1;
                            if let Some((active_act, _)) = &self.mechanics.active_alert {
                                if *active_act == step.action { self.mechanics.active_alert = None; }
                            }
                            self.push_log(format!("  -> [Mechanics] Phase {} cleared (HP Threshold Reached)", check_phase));
                        }
                    },
                    _ => {}
                }
            }
            
            if let Some(new_action) = pending_alert {
                let is_already_active = self.mechanics.active_alert.as_ref().map_or(false, |(act, _)| *act == new_action);
                if !is_already_active {
                    self.mechanics.active_alert = Some((new_action.clone(), now));
                    self.push_log(format!("  -> [Alert] YOUR TURN: {:?}", new_action));
                }
            } else {
                if let Some((_, timestamp)) = &self.mechanics.active_alert {
                    if timestamp.elapsed() >= Duration::from_secs(2) {
                        self.mechanics.active_alert = None;
                    }
                }
            }
        }
    }
}
