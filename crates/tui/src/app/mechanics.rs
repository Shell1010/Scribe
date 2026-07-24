use scribe_core::mechanics::{Action, Strategy};
use std::collections::HashMap;
use std::time::{Duration, Instant};

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

impl SpeakerMechanicsState {
    pub fn step_for_phase(
        strategy: &Strategy,
        phase: usize,
    ) -> Option<&scribe_core::mechanics::MechanicStep> {
        let opening_len = strategy.opening.as_ref().map_or(0, |o| o.len());
        if phase < opening_len {
            strategy.opening.as_ref().and_then(|o| o.get(phase))
        } else {
            match &strategy.cycle {
                Some(cycle) if !cycle.is_empty() => cycle.get((phase - opening_len) % cycle.len()),
                _ => None,
            }
        }
    }

    pub fn cycle_role_forward(&mut self) {
        if let Some(strat) = self.active_boss.get(self.strategy_index) {
            if let Some(pos) = strat
                .roles
                .iter()
                .position(|r| r == &self.selected_role || r == "all")
            {
                let new_pos = (pos + 1) % strat.roles.len();
                self.selected_role = strat.roles[new_pos].clone();
            } else if !strat.roles.is_empty() {
                self.selected_role = strat.roles[0].clone();
            }
        }
    }

    pub fn cycle_role_backward(&mut self) {
        if let Some(strat) = self.active_boss.get(self.strategy_index) {
            if let Some(pos) = strat
                .roles
                .iter()
                .position(|r| r == &self.selected_role || r == "all")
            {
                let new_pos = if pos == 0 {
                    strat.roles.len() - 1
                } else {
                    pos - 1
                };
                self.selected_role = strat.roles[new_pos].clone();
            } else if !strat.roles.is_empty() {
                self.selected_role = strat.roles[0].clone();
            }
        }
    }

    pub fn cycle_strategy_forward(&mut self) {
        if !self.active_boss.is_empty() {
            self.strategy_index = (self.strategy_index + 1) % self.active_boss.len();
            self.current_phase = 0;
            self.sync_role_to_strategy();
        }
    }

    pub fn cycle_strategy_backward(&mut self) {
        if !self.active_boss.is_empty() {
            self.strategy_index = if self.strategy_index == 0 {
                self.active_boss.len() - 1
            } else {
                self.strategy_index - 1
            };
            self.current_phase = 0;
            self.sync_role_to_strategy();
        }
    }

    fn sync_role_to_strategy(&mut self) {
        if let Some(strat) = self.active_boss.get(self.strategy_index)
            && !strat.roles.contains(&self.selected_role) && !strat.roles.is_empty() {
                self.selected_role = strat.roles[0].clone();
            }
    }
}
