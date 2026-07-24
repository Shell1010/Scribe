use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{Duration, Instant};
use scribe_core::mechanics::{Action, Trigger};
use scribe_parser::events::ScribeEvent;

pub mod dps;
pub mod events;
pub mod mechanics;

pub use dps::{DamageInstance, DpsSample, DPS_HISTORY_MAX, DPSTracker, INSTANCE_MAX};
pub use events::{
    ClassInfo, CombatMetrics, DropMetric, EntityState, EventCtx, MonsterCombatState, MAX_HISTORY,
};
pub use mechanics::SpeakerMechanicsState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchTarget {
    Log,
    PartyFilter,
    SkillFilter,
    DropFilter,
}

pub struct App {
    pub active_tab: usize,
    pub tabs: Vec<&'static str>,
    pub scroll_class_x: u16,
    pub scroll_class_y: u16,
    pub scroll_metrics_y: u16,
    pub scroll_log_x: u16,

    pub snap: bool,
    pub active_monsters: HashMap<String, MonsterCombatState>,

    pub entities: HashMap<String, EntityState>,
    pub system_log: VecDeque<String>,
    pub last_vitals: HashMap<String, (i32, i32, i32)>,
    recent_events: VecDeque<ScribeEvent>,
    log_file: Option<File>,
    pub should_quit: bool,
    pub users_in_room: Vec<String>,
    pub room_users_detailed: Vec<(String, String, i32)>,

    pub class_info: ClassInfo,
    pub selected_skill_index: usize,
    pub combat_metrics: CombatMetrics,
    pub item_cache: HashMap<u32, String>,
    pub username: String,
    pub verbose: bool,

    pub mechanics: SpeakerMechanicsState,
    pub selected_log_index: usize,
    pub status_message: Option<(String, Instant)>,

    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub search_match_idx: usize,
    pub search_mode: bool,
    pub input_buffer: String,

    pub paused: bool,

    pub split_left: u16,
    pub dragging: Option<DragSide>,
    pub selected_entity_index: usize,

    pub dps_history: VecDeque<DpsSample>,
    pub show_dps_graph: bool,
    pub dps_peak: f64,
    pub dps_peak_at: Instant,

    pub show_help: bool,
    pub party_filter: String,
    pub skill_filter: String,
    pub drop_filter: String,
    pub filter_mode: bool,
    pub filter_target: SearchTarget,
    pub highlighted_entity: Option<String>,
    pub last_alert: Option<(Action, Instant)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragSide {
    Left,
}

impl App {
    pub fn new(log_path: Option<&str>, username: String, verbose: bool) -> Self {
        let log_file = log_path.map(|path| {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("Failed to open log file")
        });

        Self {
            active_tab: 0,
            tabs: vec!["Output Log", "Class Data", "Combat Metrics", "Ultra Mechanics", "Party Status"],
            scroll_class_x: 0,
            scroll_class_y: 0,
            scroll_metrics_y: 0,
            scroll_log_x: 0,
            entities: HashMap::new(),
            system_log: VecDeque::with_capacity(1000),
            last_vitals: HashMap::new(),
            recent_events: VecDeque::with_capacity(1000),
            log_file,
            should_quit: false,
            users_in_room: Vec::new(),
            room_users_detailed: Vec::new(),
            username: username.to_lowercase(),
            verbose,
            snap: false,
            active_monsters: HashMap::new(),
            class_info: ClassInfo::default(),
            selected_skill_index: 0,
            combat_metrics: CombatMetrics::default(),
            item_cache: HashMap::new(),
            mechanics: SpeakerMechanicsState::default(),
            selected_log_index: 0,
            status_message: None,

            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_idx: 0,
            search_mode: false,
            input_buffer: String::new(),

            paused: false,

            split_left: 0,
            dragging: None,
            selected_entity_index: 0,

            dps_history: VecDeque::with_capacity(DPS_HISTORY_MAX),
            show_dps_graph: false,
            dps_peak: 0.0,
            dps_peak_at: Instant::now(),

            show_help: false,
            party_filter: String::new(),
            skill_filter: String::new(),
            drop_filter: String::new(),
            filter_mode: false,
            filter_target: SearchTarget::Log,
            highlighted_entity: None,
            last_alert: None,
        }
    }

    pub fn format_compact(val: f64) -> String {
        if val >= 1_000_000.0 {
            format!("{:.1}M", val / 1_000_000.0)
        } else if val >= 10_000.0 {
            format!("{:.1}K", val / 1_000.0)
        } else {
            format!("{:.0}", val)
        }
    }

    pub fn push_log(&mut self, message: String) {
        if let Some(file) = &mut self.log_file {
            let _ = writeln!(file, "{}", message);
        }
        if self.system_log.len() >= MAX_HISTORY {
            self.system_log.pop_front();
        }
        self.system_log.push_back(message);
        self.snap = true;
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn clear_stale_status(&mut self) {
        if let Some((_, ts)) = &self.status_message
            && ts.elapsed() >= Duration::from_secs(2) {
                self.status_message = None;
            }
    }

    pub fn clamp_log_selection(&mut self) {
        if !self.system_log.is_empty() {
            self.selected_log_index = self.selected_log_index.min(self.system_log.len() - 1);
        } else {
            self.selected_log_index = 0;
        }
    }

    pub fn recalc_search(&mut self) {
        self.search_matches.clear();
        self.search_match_idx = 0;
        if self.search_query.is_empty() {
            return;
        }
        let q = self.search_query.to_lowercase();
        self.search_matches = self
            .system_log
            .iter()
            .enumerate()
            .filter(|(_, line)| line.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
    }

    pub fn clamp_search_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.search_match_idx = self.search_match_idx.min(self.search_matches.len() - 1);
        } else {
            self.search_match_idx = 0;
        }
    }

    pub fn jump_to_match(&mut self, forward: bool) {
        if self.search_matches.is_empty() {
            return;
        }
        if forward {
            self.search_match_idx = (self.search_match_idx + 1) % self.search_matches.len();
        } else {
            self.search_match_idx = if self.search_match_idx == 0 {
                self.search_matches.len() - 1
            } else {
                self.search_match_idx - 1
            };
        }
        self.selected_log_index = self.search_matches[self.search_match_idx];
    }

    pub fn handle_event(&mut self, event: ScribeEvent) {
        if self.recent_events.len() >= MAX_HISTORY {
            self.recent_events.pop_front();
        }
        self.recent_events.push_back(event.clone());

        let mut log_buf = Vec::new();
        {
            let mut ctx = EventCtx {
                username: &self.username,
                active_monsters: &mut self.active_monsters,
                entities: &mut self.entities,
                last_vitals: &mut self.last_vitals,
                combat_metrics: &mut self.combat_metrics,
                mechanics: &mut self.mechanics,
                class_info: &mut self.class_info,
                item_cache: &mut self.item_cache,
                users_in_room: &mut self.users_in_room,
                room_users_detailed: &mut self.room_users_detailed,
                log: &mut log_buf,
            };
            events::handle_event(event, &mut ctx);
        }
        for msg in log_buf {
            self.push_log(msg);
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();

        let recent_dps = self.combat_metrics.dps.recent_group_dps();
        let total_group_dps = self.combat_metrics.dps.total_group_dps();
        let recent_solo_dps = self.combat_metrics.dps.recent_solo_dps();
        let total_solo_dps = self.combat_metrics.dps.total_solo_dps();

        if total_group_dps > 0.0 || total_solo_dps > 0.0 {
            if self.dps_history.len() >= DPS_HISTORY_MAX {
                self.dps_history.pop_front();
            }
            self.dps_history.push_back(DpsSample {
                group: total_group_dps,
                recent_group: recent_dps,
                solo: total_solo_dps,
                recent_solo: recent_solo_dps,
                at: now,
            });

            let local_max = total_group_dps
                .max(recent_dps)
                .max(total_solo_dps)
                .max(recent_solo_dps);
            if local_max > self.dps_peak {
                self.dps_peak = local_max;
                self.dps_peak_at = now;
            }
        }

        let mut pending_alert: Option<Action> = None;
        let mut advance_phase = false;

        if let Some(strategy) = self.mechanics.active_boss.get(self.mechanics.strategy_index) {
            if let Some(reactives) = &strategy.reactives {
                for reactive in reactives {
                    if reactive.role == self.mechanics.selected_role || reactive.role == "all" {
                        let mut triggered = false;

                        match &reactive.trigger {
                            Trigger::Aura { name, timer } => {
                                if let Some(expiration) = self.mechanics.active_auras.get(name) {
                                    let remaining = expiration.saturating_duration_since(now);
                                    let buffer = Duration::from_millis(*timer as u64);
                                    if remaining > Duration::ZERO && remaining <= buffer {
                                        triggered = true;
                                    }
                                }
                            }
                            Trigger::HP { value, timer } => {
                                let current_hp = self.last_vitals.values().map(|(hp, _, _)| *hp).max().unwrap_or(0);
                                let target_hp = *value as i32;

                                if current_hp > target_hp {
                                    let hp_remaining = (current_hp - target_hp) as f64;
                                    let buffer_secs = (*timer as f64) / 1000.0;
                                    let time_to_threshold = if recent_dps > 0.0 {
                                        hp_remaining / recent_dps
                                    } else {
                                        f64::MAX
                                    };

                                    if time_to_threshold <= buffer_secs || hp_remaining <= 300_000.0 {
                                        triggered = true;
                                    } else if let Some((active_act, _)) = &self.mechanics.active_alert
                                        && *active_act == reactive.action
                                            && time_to_threshold <= (buffer_secs * 2.0)
                                        {
                                            triggered = true;
                                        }
                                }
                            }
                            _ => {}
                        }

                        if triggered {
                            pending_alert = Some(reactive.action.clone());
                            break;
                        }
                    }
                }
            }

            if let Some(step) = SpeakerMechanicsState::step_for_phase(strategy, self.mechanics.current_phase) {
                let is_my_turn = step.role == self.mechanics.selected_role || step.role == "all";

                match &step.trigger {
                    Trigger::Aura { name, timer } => {
                        if let Some(expiration) = self.mechanics.active_auras.get(name) {
                            let remaining = expiration.saturating_duration_since(now);
                            let buffer = Duration::from_millis(*timer as u64);

                            if remaining > Duration::ZERO && remaining <= buffer && is_my_turn
                                && pending_alert.is_none() {
                                    pending_alert = Some(step.action.clone());
                                }
                        }
                    }
                    Trigger::HP { value, timer } => {
                        let current_hp = self.last_vitals.values().map(|(hp, _, _)| *hp).max().unwrap_or(0);
                        let target_hp = *value as i32;

                        if current_hp > target_hp {
                            let time_to_threshold = if recent_dps > 0.0 {
                                ((current_hp - target_hp) as f64) / recent_dps
                            } else {
                                f64::MAX
                            };

                            if time_to_threshold <= (*timer as f64) / 1000.0 && is_my_turn
                                && pending_alert.is_none() {
                                    pending_alert = Some(step.action.clone());
                                }
                        } else if current_hp > 0 {
                            advance_phase = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        if advance_phase {
            self.mechanics.current_phase += 1;
            if let Some(strategy) = self.mechanics.active_boss.get(self.mechanics.strategy_index)
                && let Some(step) = SpeakerMechanicsState::step_for_phase(strategy, self.mechanics.current_phase - 1)
                    && let Some((active_act, _)) = &self.mechanics.active_alert
                        && *active_act == step.action {
                            self.mechanics.active_alert = None;
                        }
            self.push_log(format!(
                "  -> [Mechanics] Phase {} cleared (HP Threshold Reached)",
                self.mechanics.current_phase - 1
            ));
        }

        if let Some(new_action) = pending_alert {
            let is_already_active = self
                .mechanics
                .active_alert
                .as_ref()
                .is_some_and(|(act, _)| *act == new_action);
            if !is_already_active {
                self.mechanics.active_alert = Some((new_action.clone(), now));
                self.last_alert = Some((new_action, now));
                self.push_log(format!(
                    "  -> [Alert] YOUR TURN: {:?}",
                    self.mechanics.active_alert.as_ref().unwrap().0
                ));
            }
        } else if let Some((_, timestamp)) = &self.mechanics.active_alert
            && timestamp.elapsed() >= Duration::from_secs(2) {
                self.mechanics.active_alert = None;
            }
    }
}
