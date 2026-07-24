use std::collections::HashMap;
use std::time::{Duration, Instant};
use scribe_core::mechanics::{Action, BossRegistry, Trigger};
use scribe_core::models::combat::StatDetails;
use scribe_parser::events::ScribeEvent;
use serde_json::{Map, Value};

use super::mechanics::SpeakerMechanicsState;

pub const MAX_HISTORY: usize = 1000;

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
    pub dps: super::dps::DPSTracker,
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
            dps: super::dps::DPSTracker::default(),
        }
    }
}

impl CombatMetrics {
    pub fn reset_farming(&mut self) {
        self.total_kills = 0;
        self.total_gold = 0;
        self.total_exp = 0;
        self.drops.clear();
        self.session_start = Instant::now();
        self.last_activity = Instant::now();
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
#[derive(Default)]
pub struct EntityState {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub shield: i32,
    pub level: i32,
    pub class: String,
    pub is_monster: bool,
    pub auras: Vec<String>,
    pub last_action: String,
    pub base_stats: Option<StatDetails>,
    pub total_stats: Option<StatDetails>,
}


impl EntityState {
    pub fn new(name: String, is_monster: bool) -> Self {
        Self { name, is_monster, ..Default::default() }
    }
}

pub struct EventCtx<'a> {
    pub username: &'a str,
    pub active_monsters: &'a mut HashMap<String, MonsterCombatState>,
    pub entities: &'a mut HashMap<String, EntityState>,
    pub last_vitals: &'a mut HashMap<String, (i32, i32, i32)>,
    pub combat_metrics: &'a mut CombatMetrics,
    pub mechanics: &'a mut SpeakerMechanicsState,
    pub class_info: &'a mut ClassInfo,
    pub item_cache: &'a mut HashMap<u32, String>,
    pub users_in_room: &'a mut Vec<String>,
    pub room_users_detailed: &'a mut Vec<(String, String, i32)>,
    pub log: &'a mut Vec<String>,
}

pub fn handle_event(event: ScribeEvent, ctx: &mut EventCtx<'_>) {
    match event {
        ScribeEvent::CombatTick { stats, auras } => handle_combat_tick(stats, auras, ctx),
        ScribeEvent::PlayerDamage { caster, target, damage } => {
            handle_player_damage(caster, target, damage, ctx)
        }
        ScribeEvent::EnemyDamage { caster: _, target, damage } => {
            handle_enemy_damage(target, damage, ctx)
        }
        ScribeEvent::MonsterReset { target, base_hp, level } => {
            handle_monster_reset(target, base_hp, level, ctx)
        }
        ScribeEvent::InventoryLoaded { items } => {
            ctx.item_cache.extend(items);
            ctx.log
                .push(format!("  -> [System] Cached {} items from inventory", ctx.item_cache.len()));
        }
        ScribeEvent::RoomUsersDetailed { users } => handle_room_users(users, ctx),
        ScribeEvent::ItemAdded { item_id, quantity, quantity_now } => {
            handle_item_added(item_id, quantity, quantity_now, ctx)
        }
        ScribeEvent::ItemDropped { item_id, item_name, quantity } => {
            handle_item_dropped(item_id, item_name, quantity, ctx)
        }
        ScribeEvent::RoomPlayersUpdate { players } => {
            *ctx.users_in_room = players;
        }
        ScribeEvent::GoldExpGained { monster_name, gold, exp, bonus_gold } => {
            handle_gold_exp(monster_name, gold, exp, bonus_gold, ctx)
        }
        ScribeEvent::PassiveAurasApplied { target, auras } => {
            handle_passive_auras(target, auras, ctx)
        }
        ScribeEvent::ClassUpdated { uid: _, class_name, category, desc, mrm } => {
            if !desc.is_empty() {
                ctx.class_info.active_skills.clear();
                ctx.class_info.passive_skills.clear();
                ctx.class_info.name = class_name.clone();
                ctx.class_info.category = category;
                ctx.class_info.desc = desc;
                ctx.class_info.mrm = mrm;
                if let Some(ent) = ctx.entities.get_mut(ctx.username) {
                    ent.class = class_name;
                }
            }
        }
        ScribeEvent::Seia { data } => {
            if ctx.class_info.active_skills.len() > 5 {
                ctx.class_info.active_skills[5] = data;
            }
        }
        ScribeEvent::SkillsLoaded { active, passive } => {
            ctx.class_info.active_skills = active;
            if ctx.class_info.passive_skills.is_empty() {
                ctx.class_info.passive_skills = passive;
            } else {
                for (i, skill) in passive.iter().enumerate() {
                    if let Some(existing) = ctx.class_info.passive_skills.get_mut(i) {
                        existing.extend(skill.clone());
                    } else {
                        ctx.class_info.passive_skills.push(skill.clone());
                    }
                }
            }
            ctx.log.push(format!(
                "  -> Loaded {} Active & {} Passive Skills",
                ctx.class_info.active_skills.len(),
                ctx.class_info.passive_skills.len()
            ));
        }
        ScribeEvent::UserDataInitialized { username, uid, class_name, access_level: _, level } => {
            ctx.log.push(format!(
                "  -> User data initialized: {} (UID: {}, Class: {}, Lv{})",
                username, uid, class_name, level
            ));
            let ent = ctx
                .entities
                .entry(username.clone())
                .or_insert_with(|| EntityState::new(username, false));
            ent.class = class_name;
            if level > 0 {
                ent.level = level;
            }
        }
        ScribeEvent::BossAction { caster, target, message, action_type } => {
            handle_boss_action(caster, target, message, action_type, ctx)
        }
        ScribeEvent::StatUpdate { stats } => handle_stat_update(stats, ctx),
        ScribeEvent::Death { victim, killer } => {
            ctx.log.push(format!("  -> [DEATH] {} was killed by {}", victim, killer));
        }
        ScribeEvent::ZoneChange { room } => handle_zone_change(room, ctx),
        ScribeEvent::StateChange { username, state, level, entity_id, max_hp, max_mp } => {
            let status = match state {
                2 => "Entered Combat",
                1 => "Alive",
                0 => "Fainted",
                _ => "Unknown",
            };
            ctx.log.push(format!(
                "[State] {} (Lvl: {}, ID: {}) is now: {}",
                username, level, entity_id, status
            ));
            let ent = ctx
                .entities
                .entry(username.clone())
                .or_insert_with(|| EntityState::new(username, false));
            ent.level = level;
            if let Some(v) = max_hp { ent.max_hp = v; }
            if let Some(v) = max_mp { ent.max_mp = v; }
        }
        ScribeEvent::Unknown { data } => {
            ctx.log.push(format!("  -> [?] Unknown event: {}", data));
        }
    }
}

fn handle_player_damage(caster: String, target: String, damage: i32, ctx: &mut EventCtx<'_>) {
    if caster == ctx.username {
        let state = ctx.active_monsters.entry(target.clone()).or_default();
        state.damage_dealt += damage;
        ctx.combat_metrics.dps.record_solo_damage(damage);
        ctx.log.push(format!("  -> [Combat] Target: {} | Your DMG: {}", target, damage));
    }
    ctx.log
        .push(format!("  -> [Combat] Caster: {} | Target: {} | DMG: {}", caster, target, damage));
}

fn handle_enemy_damage(target: String, damage: i32, ctx: &mut EventCtx<'_>) {
    let state = ctx.active_monsters.entry(target.clone()).or_default();
    state.damage_dealt += damage;
    ctx.combat_metrics.dps.record_solo_damage(damage);
    ctx.log
        .push(format!("  -> [Combat] Target: {} | Your DMG: {}", target, damage));
}

fn handle_monster_reset(target: String, base_hp: i32, level: i32, ctx: &mut EventCtx<'_>) {
    ctx.mechanics.current_phase = 0;
    ctx.mechanics.active_alert = None;
    ctx.mechanics.last_prompt.clear();
    ctx.last_vitals.remove(&target);

    let ent = ctx
        .entities
        .entry(target.clone())
        .or_insert_with(|| EntityState::new(target.clone(), true));
    ent.max_hp = base_hp;
    ent.hp = base_hp;
    ent.level = level;

    if let Some(state) = ctx.active_monsters.get(&target) {
        let contribution = if state.total_damage_dealt > 0 {
            (state.damage_dealt as f64 / state.total_damage_dealt as f64) * 100.0
        } else {
            0.0
        };
        let hp_pct = if base_hp > 0 {
            (state.damage_dealt as f64 / base_hp as f64) * 100.0
        } else {
            0.0
        };
        ctx.log.push(format!(
            "  -> [Combat] Target: {} | Base HP: {} | Your Dmg: {} | Pct: {:.1}% | HP Pct: {:.1}%",
            target, state.base_hp, state.damage_dealt, contribution, hp_pct
        ));
    }

    ctx.active_monsters
        .insert(target, MonsterCombatState { base_hp, damage_dealt: 0, total_damage_dealt: 0 });
}

fn handle_room_users(users: Vec<(String, String, i32)>, ctx: &mut EventCtx<'_>) {
    *ctx.users_in_room = users.iter().map(|(name, _, _)| name.clone()).collect();
    *ctx.room_users_detailed = users;
    ctx.entities
        .retain(|_, e| e.is_monster || ctx.users_in_room.iter().any(|u| u == &e.name));
    for (name, class, level) in ctx.room_users_detailed.iter() {
        let ent = ctx
            .entities
            .entry(name.clone())
            .or_insert_with(|| EntityState::new(name.clone(), false));
        ent.class = class.clone();
        ent.level = *level;
    }
}

fn handle_item_added(item_id: u32, quantity: u32, quantity_now: u32, ctx: &mut EventCtx<'_>) {
    ctx.combat_metrics.reset_farming();
    let item_name = ctx
        .item_cache
        .get(&item_id)
        .cloned()
        .unwrap_or_else(|| format!("Item #{}", item_id));
    ctx.log
        .push(format!("  -> [Bag] Added {}x {} (Total in Bag: {})", quantity, item_name, quantity_now));

    let entry =
        ctx.combat_metrics.drops.entry(item_name.clone()).or_insert(DropMetric {
            id: item_id,
            name: item_name,
            total_quantity: 0,
            drop_count: 0,
            kills_at_last_drop: ctx.combat_metrics.total_kills,
        });
    entry.total_quantity += quantity;
    entry.drop_count += 1;
    entry.kills_at_last_drop = ctx.combat_metrics.total_kills;
}

fn handle_item_dropped(item_id: u32, item_name: String, quantity: u32, ctx: &mut EventCtx<'_>) {
    ctx.item_cache.insert(item_id, item_name.clone());
    let current_kills = ctx.combat_metrics.total_kills;
    let entry =
        ctx.combat_metrics.drops.entry(item_name.clone()).or_insert(DropMetric {
            id: item_id,
            name: item_name.clone(),
            total_quantity: 0,
            drop_count: 0,
            kills_at_last_drop: current_kills,
        });
    entry.total_quantity += quantity;
    entry.drop_count += 1;
    entry.kills_at_last_drop = current_kills;
    ctx.log
        .push(format!("  -> [Drop] {}x {} (ID: {})", quantity, item_name, item_id));
    ctx.combat_metrics.reset_farming();
}

fn handle_gold_exp(monster_name: String, gold: i32, exp: i32, bonus_gold: i32, ctx: &mut EventCtx<'_>) {
    let bonus = if bonus_gold > 0 { format!(" (+{} bonus)", bonus_gold) } else { String::new() };
    ctx.log.push(format!(
        "  -> [Loot] Killed {}: {} Gold{} | {} Exp",
        monster_name, gold, bonus, exp
    ));
    ctx.combat_metrics.total_kills += 1;
    ctx.combat_metrics.total_gold += gold + bonus_gold;
    ctx.combat_metrics.total_exp += exp;
}

fn handle_passive_auras(
    target: String,
    auras: Vec<(String, StatDetails)>,
    ctx: &mut EventCtx<'_>,
) {
    for (name, details) in auras {
        let readable_stats = details.to_formatted_list();
        let stats_log = if readable_stats.is_empty() {
            String::new()
        } else {
            format!(" ({})", readable_stats.join(", "))
        };

        ctx.log
            .push(format!("  -> [Passives] {} applied to {}{}", name, target, stats_log));

        if let Ok(value) = serde_json::to_value(details)
            && let Ok(mut stats_map) =
                serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(value)
        {
            stats_map.retain(|_, v| !v.is_null());

            if !readable_stats.is_empty() {
                stats_map
                    .insert("stats".to_string(), serde_json::Value::String(readable_stats.join(", ")));
            }

            if let Some(skill) = ctx.class_info.passive_skills.iter_mut().find(|s| {
                s.get("nam").and_then(|v| v.as_str()) == Some(name.as_str())
            }) {
                skill.extend(stats_map);
            } else {
                stats_map.insert("nam".to_string(), serde_json::Value::String(name));
                ctx.class_info.passive_skills.push(stats_map);
            }
        }
    }
}

fn handle_boss_action(
    caster: String,
    target: String,
    message: String,
    action_type: String,
    ctx: &mut EventCtx<'_>,
) {
    let msg_part = if message.is_empty() { String::new() } else { format!(" | msg: {}", message) };
    ctx.log
        .push(format!("  -> [Boss] {} >> {} : ({}){}", caster, target, action_type, msg_part));

    let now = Instant::now();
    let is_duplicate = ctx.mechanics.last_prompt == message
        && now.duration_since(ctx.mechanics.last_prompt_time) < Duration::from_millis(500);

    if is_duplicate {
        return;
    }

    let msg_lower = message.to_lowercase();

    enum BossAction {
        Reactive { action: Action, is_mine: bool, role: String },
        PhaseMatch { step: scribe_core::mechanics::MechanicStep, new_phase: usize },
        Desync,
        None,
    }

    let boss_result = ctx
        .mechanics
        .active_boss
        .get(ctx.mechanics.strategy_index)
        .map(|strategy| {
            if let Some(reactives) = &strategy.reactives {
                for reactive in reactives {
                    if let Trigger::Message(msg) = &reactive.trigger
                        && msg_lower.contains(&msg.to_lowercase()) {
                            let is_mine = reactive.role == ctx.mechanics.selected_role
                                || reactive.role == "all";
                            return BossAction::Reactive {
                                action: reactive.action.clone(),
                                is_mine,
                                role: reactive.role.clone(),
                            };
                        }
                }
            }

            for offset in 0..=2 {
                if let Some(s) =
                    SpeakerMechanicsState::step_for_phase(strategy, ctx.mechanics.current_phase + offset)
                    && let Trigger::Message(ref prompt) = s.trigger
                        && msg_lower.contains(&prompt.to_lowercase()) {
                            return BossAction::PhaseMatch {
                                step: s.clone(),
                                new_phase: ctx.mechanics.current_phase + offset + 1,
                            };
                        }
            }

            if msg_lower.contains("truth")
                || msg_lower.contains("listen")
                || msg_lower.contains("equal")
            {
                BossAction::Desync
            } else {
                BossAction::None
            }
        })
        .unwrap_or(BossAction::None);

    match boss_result {
        BossAction::Reactive { action, is_mine, role } => {
            if is_mine {
                ctx.mechanics.active_alert = Some((action.clone(), now));
                ctx.log.push(format!("  -> [Reactive] YOUR TURN: {:?}", action));
            } else {
                ctx.mechanics.active_alert = None;
                ctx.log
                    .push(format!("  -> [Reactive] {}'s turn to {:?}", role, action));
            }
        }
        BossAction::PhaseMatch { step, new_phase } => {
            ctx.mechanics.last_prompt = message;
            ctx.mechanics.last_prompt_time = now;
            ctx.mechanics.current_phase = new_phase;

            if step.role == ctx.mechanics.selected_role || step.role == "all" {
                ctx.mechanics.active_alert = Some((step.action.clone(), now));
                ctx.log.push(format!(
                    "  -> [Speaker] Phase {}! YOUR TURN: {:?}",
                    ctx.mechanics.current_phase, step.action
                ));
            } else {
                ctx.mechanics.active_alert = None;
                ctx.log.push(format!(
                    "  -> [Speaker] Phase {} ({}'s turn to {:?})",
                    ctx.mechanics.current_phase, step.role, step.action
                ));
            }
        }
        BossAction::Desync => {
            ctx.log.push(format!(
                "  -> [Mechanic Desync] Tracker at {}, Boss said: {}",
                ctx.mechanics.current_phase, msg_lower
            ));
        }
        BossAction::None => {}
    }
}

fn handle_stat_update(stats: StatDetails, ctx: &mut EventCtx<'_>) {
    let mut changed = Vec::new();
    let mut add = |name: &str, val: Option<f64>| {
        if let Some(v) = val {
            changed.push(format!("{}: {}", name, v));
        }
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
        ctx.log.push(format!("  -> [Stats] Updated: {}", changed.join(", ")));
    }

    let ent = ctx
        .entities
        .entry(ctx.username.to_string())
        .or_insert_with(|| EntityState::new(ctx.username.to_string(), false));
    ent.base_stats = Some(StatDetails {
        base_str: stats.base_str,
        base_dex: stats.base_dex,
        base_int: stats.base_int,
        base_end: stats.base_end,
        base_wis: stats.base_wis,
        base_lck: stats.base_lck,
        ..Default::default()
    });
    ent.total_stats = Some(stats);
}

fn handle_zone_change(room: String, ctx: &mut EventCtx<'_>) {
    ctx.log.push(format!("\n>>> ROOM SHIFT: {} <<<\n", room));
    ctx.last_vitals.clear();
    ctx.entities.clear();
    ctx.mechanics.current_phase = 0;
    ctx.mechanics.active_alert = None;
    ctx.mechanics.last_prompt.clear();
    ctx.mechanics.active_boss.clear();

    match std::fs::read_to_string("./mechanics.json") {
        Ok(json_str) => match serde_json::from_str::<BossRegistry>(&json_str) {
            Ok(registry) => {
                let room_lower = room.to_lowercase();
                if let Some((boss_key, strategies)) =
                    registry.iter().find(|(key, _)| room_lower.contains(*key))
                {
                    ctx.mechanics.active_boss = strategies.clone();
                    ctx.mechanics.strategy_index = 0;
                    ctx.log
                        .push(format!("  -> [Mechanics] Loaded strategies for: {}", boss_key));
                }
            }
            Err(e) => {
                ctx.log
                    .push(format!("  -> [Mechanics Error] Failed to parse mechanics.json: {}", e));
            }
        },
        Err(e) => {
            ctx.log.push(format!(
                "  -> [Mechanics Error] Could not read mechanics.json: {}",
                e
            ));
        }
    }
}

fn format_aura_metadata(aura: &scribe_parser::events::AuraTimelineDelta) -> String {
    let mut parts = Vec::new();
    if let Some(d) = aura.duration { parts.push(format!("Dur: {}s", d)); }
    if let Some(v) = aura.value { parts.push(format!("Val: {}", v)); }
    if parts.is_empty() { String::new() } else { format!(" [{}]", parts.join(" | ")) }
}

fn handle_combat_tick(
    stats: Vec<scribe_parser::events::StatTimelineDelta>,
    auras: Vec<scribe_parser::events::AuraTimelineDelta>,
    ctx: &mut EventCtx<'_>,
) {
    for stat in stats {
        let mut updates = Vec::new();
        let (prev_hp, prev_mp, prev_sh) = ctx
            .last_vitals
            .get(&stat.target)
            .copied()
            .unwrap_or((stat.hp.unwrap_or(0), stat.mp.unwrap_or(0), stat.shield.unwrap_or(0)));

        if let Some(new_hp) = stat.hp {
            let diff = new_hp - prev_hp;
            if diff != 0 {
                let label = if diff > 0 { "Heal" } else { "Dmg" };
                if diff < 0
                    && let Some(state) = ctx.active_monsters.get_mut(&stat.target) {
                        let dmg = diff.abs();
                        state.total_damage_dealt += dmg;
                        ctx.combat_metrics.dps.record_group_damage(new_hp, dmg);
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
                if diff < 0
                    && let Some(state) = ctx.active_monsters.get_mut(&stat.target) {
                        let dmg = diff.abs();
                        state.total_damage_dealt += dmg;
                        ctx.combat_metrics.dps.record_group_damage(new_sh, dmg);
                    }
                updates.push(format!("{}: {} ({})", label, diff.abs(), new_sh));
            }
        }

        ctx.last_vitals.insert(
            stat.target.clone(),
            (
                stat.hp.unwrap_or(prev_hp),
                stat.mp.unwrap_or(prev_mp),
                stat.shield.unwrap_or(prev_sh),
            ),
        );

        if !updates.is_empty() {
            ctx.log
                .push(format!("  -> [Vitals] {} -> {}", stat.target, updates.join(" | ")));
        }

        let ent = ctx
            .entities
            .entry(stat.target.clone())
            .or_insert_with(|| EntityState::new(stat.target.clone(), stat.is_monster));
        if let Some(v) = stat.hp {
            ent.hp = v;
            if v > ent.max_hp { ent.max_hp = v; }
        }
        if let Some(v) = stat.mp {
            ent.mp = v;
            if v > ent.max_mp { ent.max_mp = v; }
        }
        if let Some(v) = stat.shield { ent.shield = v; }
    }

    for aura in auras {
        let meta_str = format_aura_metadata(&aura);
        match aura.action.as_str() {
            "Applied" | "Refreshed" => {
                if let Some(d) = aura.duration {
                    let dur_secs = d as f64;
                    ctx.mechanics.active_auras.insert(
                        aura.aura_name.clone(),
                        Instant::now() + Duration::from_secs_f64(dur_secs),
                    );
                }
                let symbol = if aura.action == "Applied" { "+" } else { "*" };
                ctx.log.push(format!(
                    "  -> [{}] [Aura] {} {} on {}{}",
                    symbol, aura.aura_name, aura.action, aura.target, meta_str
                ));
                if let Some(ent) = ctx.entities.get_mut(&aura.target)
                    && !ent.auras.contains(&aura.aura_name) {
                        ent.auras.push(aura.aura_name.clone());
                    }

                let should_advance = ctx
                    .mechanics
                    .active_boss
                    .get(ctx.mechanics.strategy_index)
                    .and_then(|s| SpeakerMechanicsState::step_for_phase(s, ctx.mechanics.current_phase))
                    .and_then(|step| {
                        if let Trigger::Aura { name, .. } = &step.trigger
                            && name.eq_ignore_ascii_case(&aura.aura_name) {
                                return Some(step.action.clone());
                            }
                        None
                    });

                if let Some(action) = should_advance {
                    ctx.mechanics.current_phase += 1;
                    if let Some((active_act, _)) = &ctx.mechanics.active_alert
                        && *active_act == action { ctx.mechanics.active_alert = None; }
                    ctx.log.push(format!(
                        "  -> [Mechanics] Phase {} cleared ({} refreshed)",
                        ctx.mechanics.current_phase - 1, aura.aura_name
                    ));
                }
            }
            "Faded" => {
                ctx.mechanics.active_auras.remove(&aura.aura_name);
                ctx.log.push(format!(
                    "  -> [-] [Aura] {} expired from {}{}",
                    aura.aura_name, aura.target, meta_str
                ));
                if let Some(ent) = ctx.entities.get_mut(&aura.target) {
                    ent.auras.retain(|a| a != &aura.aura_name);
                }
            }
            _ => ctx.log.push(format!(
                "  -> [?] [Aura] {} {} on {}{}",
                aura.aura_name, aura.action, aura.target, meta_str
            )),
        }
    }
}
