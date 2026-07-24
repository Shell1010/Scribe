use std::time::{Duration, Instant};

pub type DamageInstance = (i32, Instant);
pub const INSTANCE_MAX: usize = 20;
pub const DPS_HISTORY_MAX: usize = 200;

#[derive(Debug, Clone)]
pub struct DpsSample {
    pub group: f64,
    pub recent_group: f64,
    pub solo: f64,
    pub recent_solo: f64,
    pub at: Instant,
}

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

impl DPSTracker {
    pub fn recent_group_dps(&self) -> f64 {
        if self.damage_instances.len() < INSTANCE_MAX / 2 {
            return 0.0;
        }
        let first = &self.damage_instances[0];
        let last = &self.damage_instances[self.damage_instances.len() - 1];
        let hp_diff = (first.0 - last.0) as f64;
        let time_diff = last.1.duration_since(first.1).as_secs_f64();

        if time_diff > 0.0 && hp_diff > 0.0 {
            hp_diff / time_diff
        } else {
            0.0
        }
    }

    pub fn recent_solo_dps(&self) -> f64 {
        match (
            self.solo_damage_instances.first(),
            self.solo_damage_instances.last(),
        ) {
            (Some(first), Some(last)) if self.solo_damage_instances.len() >= 2 => {
                let total_dmg: i32 = self.solo_damage_instances.iter().map(|(dmg, _)| dmg).sum();
                let time_diff = last.1.duration_since(first.1).as_secs_f64();
                if time_diff > 0.0 {
                    total_dmg as f64 / time_diff.max(0.5)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    pub fn total_group_dps(&self) -> f64 {
        let secs = self.session_start.elapsed().as_secs_f64().max(1.0);
        self.group_total_damage as f64 / secs
    }

    pub fn total_solo_dps(&self) -> f64 {
        let secs = self.session_start.elapsed().as_secs_f64().max(1.0);
        self.solo_total_damage as f64 / secs
    }

    pub fn record_group_damage(&mut self, hp_value: i32, damage: i32) {
        self.update();
        if self.damage_instances.len() >= INSTANCE_MAX {
            self.damage_instances.remove(0);
        }
        let now = Instant::now();
        self.damage_instances.push((hp_value, now));
        self.group_total_damage += damage;
        self.instance_time = now - self.damage_instances[0].1;
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_activity) > Duration::from_secs(60) {
            *self = Self::default();
        } else {
            self.last_activity = now;
        }
    }

    pub fn record_solo_damage(&mut self, dmg: i32) {
        self.update();
        if self.solo_damage_instances.len() >= INSTANCE_MAX {
            self.solo_damage_instances.remove(0);
        }
        self.solo_damage_instances.push((dmg, Instant::now()));
        self.solo_total_damage += dmg;
    }
}
