pub mod events;
use scribe_core::IdentityMapper;
use scribe_core::models::{SfsEnvelope, SfsContent};
use events::{AuraTimelineDelta, StatTimelineDelta};
pub use events::ScribeEvent;
pub struct ScribeParser {
    identity_mapper: IdentityMapper,
    monster_levels: std::collections::HashMap<u32, i32>,
}

impl ScribeParser {
    pub fn new(identity_mapper: IdentityMapper) -> Self {
        Self {
            identity_mapper,
            monster_levels: std::collections::HashMap::new(),
        }
    }

    pub fn parse_packet(&mut self, raw_json: &str) -> Vec<ScribeEvent> {
        let mut output = Vec::new();
        let envelope: SfsEnvelope = match serde_json::from_str(raw_json) {
            Ok(env) => env,
            Err(_e) => {
                if cfg!(debug_assertions) {
                    output.push(ScribeEvent::Unknown { data: raw_json.to_string() });
                };
                return output
            }
        };

        match envelope.b.o {

            SfsContent::Mtls(payload) => {
                let target_token = format!("m:{}", payload.id);
                output.push(ScribeEvent::MonsterReset {
                    target: self.identity_mapper.resolve_actor(&target_token),
                    base_hp: payload.o.int_hp.unwrap_or_default(),
                    level: self.monster_levels.get(&(payload.id as u32)).copied().unwrap_or(0),
                });
            }

            SfsContent::Who(payload) => {
                let users: Vec<(String, String, i32)> = payload.users.values()
                    .map(|u| (u.name.to_lowercase(), u.class.clone(), u.level))
                    .collect();
                output.push(ScribeEvent::RoomUsersDetailed { users });
            }

            SfsContent::LoadInventoryBig(payload) => {
                // Move names directly out of the payload instead of cloning.
                let items_map = payload.items.into_iter()
                    .map(|item| (item.item_id, item.s_name))
                    .collect();
                output.push(ScribeEvent::InventoryLoaded { items: items_map });
            }

            SfsContent::AddItems(payload) => {
                for (id_str, details) in payload.items {
                    if let Ok(item_id) = id_str.parse::<u32>() {
                        output.push(ScribeEvent::ItemAdded {
                            item_id,
                            quantity: details.i_qty,
                            quantity_now: details.i_qty_now,
                        });
                    }
                }
            }

            SfsContent::DropItem(payload) => {
                for (_, item) in payload.items {
                    output.push(ScribeEvent::ItemDropped {
                        item_id: item.item_id,
                        item_name: item.s_name,
                        quantity: item.i_qty,
                    });
                }
            }

            SfsContent::Seia(payload) => {
                output.push(ScribeEvent::Seia { data: payload.o });
            }

            SfsContent::UpdateClass(payload) => {
                let category = match payload.class_cat.as_str() {
                    "M1" => "Tank",
                    "M2" => "Dodge Melee",
                    "M3" => "Hybrid",
                    "M4" => "Power Melee",
                    "C1" => "Offensive Caster",
                    "C2" => "Defensive Caster",
                    "C3" => "Power Caster",
                    "S1" => "Luck Hybrid",
                    other => other,
                };

                output.push(ScribeEvent::ClassUpdated {
                    uid: payload.uid,
                    class_name: payload.class_name,
                    category: category.to_string(),
                    desc: payload.desc,
                    mrm: payload.mrm,
                });
            }

            SfsContent::SAct(payload) => {
                output.push(ScribeEvent::SkillsLoaded {
                    active: payload.actions.active,
                    passive: payload.actions.passive,
                });
            }

            SfsContent::AuraPlusP(payload) => {
                let target = self.identity_mapper.resolve_actor(&payload.target_info);
                let auras = payload.auras.into_iter()
                    .map(|a| {
                        let stats = a.to_stat_details();
                        (a.nam, stats)
                    })
                    .collect();
                output.push(ScribeEvent::PassiveAurasApplied { target, auras });
            }

            SfsContent::AddGoldExp(payload) => {
                let target_token = format!("m:{}", payload.id);
                output.push(ScribeEvent::GoldExpGained {
                    monster_name: self.identity_mapper.resolve_actor(&target_token),
                    gold: payload.gold,
                    exp: payload.exp,
                    bonus_gold: payload.bonus_gold,
                });
            }

            SfsContent::MoveToArea(payload) => {
                self.identity_mapper.set_current_map(&payload.map_name);

                // Register players by iterating by reference — no clone needed.
                for p in &payload.uo_branch {
                    if let Some(id) = p.ent_id {
                        self.identity_mapper.register_player(id, &p.username);
                    }
                }

                // Build lookup dict by moving mondef entries (no clone).
                let dict: std::collections::HashMap<u32, String> = payload.mondef
                    .into_iter()
                    .map(|m| (m.mon_id, m.mon_name))
                    .collect();

                // register_monster now takes &str — no format! allocation for id.
                for m in &payload.mon_branch {
                    if let Some(name) = dict.get(&m.mon_id) {
                        let token = format!("m:{}", m.mon_map_id);
                        self.identity_mapper.register_monster(&payload.map_name, &token, name);
                    }
                    if let Some(lvl) = m.level {
                        self.monster_levels.insert(m.mon_map_id, lvl);
                    }
                }

                output.push(ScribeEvent::RoomPlayersUpdate {
                    players: payload.uo_branch.into_iter().map(|p| p.username.to_lowercase()).collect(),
                });
                output.push(ScribeEvent::ZoneChange { room: payload.map_name });
            }

            SfsContent::UpdateGuild(payload) => {
                // Iterate by reference — register_player only needs &str.
                for member in &payload.guild.user_list {
                    self.identity_mapper.register_player(member.id, &member.username);
                }
            }

            SfsContent::PlayerDeath(payload) => {
                let victim_token = format!("p:{}", payload.user_id);
                let killer_token = format!("{}:{}", payload.ent_type, payload.destroyer_id);

                output.push(ScribeEvent::Death {
                    victim: self.identity_mapper.resolve_actor(&victim_token),
                    killer: self.identity_mapper.resolve_actor(&killer_token),
                });
            }

            SfsContent::StatUpdate(payload) => {
                output.push(ScribeEvent::StatUpdate { stats: payload.sta });
            }

            SfsContent::UserState(payload) => {
                if let Some(ent_id) = payload.ent_id {
                    self.identity_mapper.register_player(ent_id, &payload.username);

                    output.push(ScribeEvent::StateChange {
                        username: payload.username.to_lowercase(),
                        state: payload.state_data.int_state.unwrap_or(1),
                        entity_id: ent_id,
                        level: payload.level.unwrap_or(100),
                        max_hp: payload.state_data.max_hp,
                        max_mp: payload.state_data.max_mp,
                    });
                }
            }

            SfsContent::InitUserData(payload) => {
                self.identity_mapper.register_player(payload.uid, &payload.data.username);
                self.identity_mapper.register_staff_level(payload.uid, payload.data.access_level);

                output.push(ScribeEvent::UserDataInitialized {
                    username: payload.data.username.to_lowercase(),
                    uid: payload.uid,
                    access_level: payload.data.access_level,
                    class_name: payload.data.class_name.unwrap_or_else(|| "Unknown Class".to_string()),
                    level: payload.data.level.unwrap_or(0),
                });
            }

            SfsContent::InitUserDatas(payload) => {
                for data in payload.a {
                    self.identity_mapper.register_player(data.uid, &data.data.username);
                    self.identity_mapper.register_staff_level(data.uid, data.data.access_level);

                    output.push(ScribeEvent::UserDataInitialized {
                        username: data.data.username.to_lowercase(),
                        uid: data.uid,
                        access_level: data.data.access_level,
                        class_name: data.data.class_name.unwrap_or_else(|| "Unknown Class".to_string()),
                        level: data.data.level.unwrap_or(0),
                    });
                }
            }

            SfsContent::Combat(payload) => {
                let mut stats_deltas = Vec::new();
                let mut aura_deltas = Vec::new();

                for (monster_idx, data) in payload.m {
                    let resolved = self.identity_mapper.resolve_actor(&format!("m:{}", monster_idx));
                    if data.hp.is_some() || data.shield.is_some() {
                        stats_deltas.push(StatTimelineDelta {
                            target: resolved,
                            hp: data.hp,
                            mp: None,
                            shield: data.shield,
                            is_monster: true,
                        });
                    }
                }

                for sarsa in payload.sarsa {
                    let caster = self.identity_mapper.resolve_actor(&sarsa.c_inf);
                    for action in sarsa.a {
                        if let Some(act_type) = &action.action_type {
                            if (act_type == "hit" || act_type == "crit" || act_type == "d") && action.hp > 0 {
                                output.push(ScribeEvent::EnemyDamage {
                                    caster: caster.clone(),
                                    target: self.identity_mapper.resolve_actor(&action.t_inf),
                                    damage: action.hp,
                                });
                            }
                        }
                    }
                }

                for sara in payload.sara {
                    let action = sara.action_result;
                    if let Some(act_type) = &action.action_type {
                        if (act_type == "hit" || act_type == "crit" || act_type == "d") && action.hp > 0 {
                            output.push(ScribeEvent::PlayerDamage {
                                caster: self.identity_mapper.resolve_actor(&action.c_inf),
                                target: self.identity_mapper.resolve_actor(&action.t_inf),
                                damage: action.hp,
                            });
                        }
                    }
                }

                for (username, data) in payload.p {
                    if data.hp.is_some() || data.mp.is_some() {
                        stats_deltas.push(StatTimelineDelta {
                            target: username.to_lowercase(),
                            hp: data.hp,
                            mp: data.mp,
                            shield: None,
                            is_monster: false,
                        });
                    }
                }

                if !payload.anims.is_empty() {
                    for anim in payload.anims {
                        if let Some(message_text) = anim.msg && let Some(anim_str) = anim.anim_str {
                            output.push(ScribeEvent::BossAction {
                                caster: self.identity_mapper.resolve_actor(&anim.c_inf),
                                target: self.identity_mapper.resolve_actor(&anim.t_inf),
                                message: message_text,
                                action_type: anim_str,
                            });
                        }
                    }
                }

                for aura_event in payload.a {
                    // Resolve once per aura event, not per aura detail.
                    let caster_name = aura_event.caster
                        .as_deref()
                        .map(|c| self.identity_mapper.resolve_actor(c))
                        .unwrap_or_else(|| "System".to_string());
                    let target_name = self.identity_mapper.resolve_actor(&aura_event.target);

                    let action_type = match aura_event.cmd.as_str() {
                        "aura+" => "Applied",
                        "aura++" => "Refreshed",
                        "aura-" => "Faded",
                        other => other,
                    }.to_string();

                    // Process single aura detail.
                    if let Some(details) = aura_event.aura {
                        aura_deltas.push(AuraTimelineDelta {
                            action: action_type.clone(),
                            caster: caster_name.clone(),
                            target: target_name.clone(),
                            aura_name: details.name,
                            value: details.val,
                            duration: details.duration,
                            aura_type: details.aura_type,
                            is_new: details.is_new,
                        });
                    }

                    // Process bulk aura details — reuse resolved names.
                    if let Some(bulk) = aura_event.auras {
                        for details in bulk {
                            aura_deltas.push(AuraTimelineDelta {
                                action: action_type.clone(),
                                caster: caster_name.clone(),
                                target: target_name.clone(),
                                aura_name: details.name,
                                value: details.val,
                                duration: details.duration,
                                aura_type: details.aura_type,
                                is_new: details.is_new,
                            });
                        }
                    }
                }

                output.push(ScribeEvent::CombatTick {
                    stats: stats_deltas,
                    auras: aura_deltas,
                })
            }
        };
        output
    }
}
