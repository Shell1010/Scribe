pub mod events;
use scribe_core::IdentityMapper;
use scribe_core::models::{SfsEnvelope, SfsContent};
use events::{AuraTimelineDelta, StatTimelineDelta};
pub use events::ScribeEvent;
pub struct ScribeParser {
    identity_mapper: IdentityMapper,
}

impl ScribeParser {

    pub fn new(identity_mapper: IdentityMapper) -> Self {
        Self { identity_mapper }
    }


    pub fn parse_packet(&self, raw_json: &str) -> Vec<ScribeEvent> {
        let mut output = Vec::new();
        let envelope: SfsEnvelope = match serde_json::from_str(raw_json) {
            Ok(env) => env,
            Err(_e) => {
                if cfg!(debug_assertions) {
                    // println!("{}", raw_json);
                    output.push(ScribeEvent::Unknown { data: raw_json.to_string()});
                };
                return output
            }
        };

        match envelope.b.o {
            SfsContent::Seia(payload) => {
                output.push(ScribeEvent::Seia {
                    data: payload.o
                });
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
                        .map(|a| (a.nam.clone(), a.to_stat_details()))
                        .collect();
                    
                    output.push(ScribeEvent::PassiveAurasApplied { target, auras });
            }
            
            SfsContent::AddGoldExp(payload) => {
                
                let target_token = format!("m:{}", payload.id);
                let monster_name = self.identity_mapper.resolve_actor(&target_token);
            
                output.push(ScribeEvent::GoldExpGained {
                    monster_name,
                    gold: payload.gold,
                    exp: payload.exp,
                    bonus_gold: payload.bonus_gold,
                });
            }
            SfsContent::MoveToArea(payload) => {
                self.identity_mapper.set_current_map(&payload.map_name);
                for p in &payload.uo_branch {
                    if let Some(id) = p.ent_id {
                        self.identity_mapper.register_player(id, &p.username);
                    }
                }
                let dict: std::collections::HashMap<_, _> = payload.mondef.into_iter().map(|m| (m.mon_id, m.mon_name)).collect();
                for m in payload.mon_branch {
                    if let Some(name) = dict.get(&m.mon_id) {
                        self.identity_mapper.register_monster(&payload.map_name, format!("m:{}", m.mon_map_id), name);
                    }
                }
                output.push(ScribeEvent::RoomPlayersUpdate { players: payload.uo_branch.iter().map(|p| p.username.clone()).collect() });
                output.push(ScribeEvent::ZoneChange { room: payload.map_name });
            }

            SfsContent::UpdateGuild(payload) => {
                for member in payload.guild.user_list.clone().into_iter() {
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
                output.push(ScribeEvent::StatUpdate {
                    stats: payload.sta,
                });
            }
            SfsContent::UserState(payload) => {
                if let Some(ent_id) = payload.ent_id {
                    self.identity_mapper.register_player(ent_id, &payload.username);
                }

                if let Some(ent_id) = payload.ent_id {
                    let other_event = ScribeEvent::StateChange {
                        username: payload.username.clone(),
                        state: payload.state_data.int_state.unwrap_or(1),
                        entity_id: ent_id,
                        level: payload.level.unwrap_or(100),
                    };
                    output.push(other_event);
                }


            }

            SfsContent::InitUserData(payload) => {
                self.identity_mapper.register_player(payload.uid, &payload.data.username);
                self.identity_mapper.register_staff_level(payload.uid, payload.data.access_level);
            
                output.push(ScribeEvent::UserDataInitialized {
                    username: payload.data.username,
                    uid: payload.uid,
                    access_level: payload.data.access_level,
                    class_name: payload.data.class_name.unwrap_or_else(|| "Unknown Class".to_string()),
                });
            }

            SfsContent::InitUserDatas(payload) => {
                for data in payload.a {
                    self.identity_mapper.register_player(data.uid, &data.data.username);
                    self.identity_mapper.register_staff_level(data.uid, data.data.access_level);

                    output.push(ScribeEvent::UserDataInitialized {
                        username: data.data.username,
                        uid: data.uid,
                        access_level: data.data.access_level,
                        class_name: data.data.class_name.unwrap_or_else(|| "Unknown Class".to_string()),
                    });
                }
            
            }

            SfsContent::Combat(payload) => {
                let mut stats_deltas = Vec::new();
                let mut aura_deltas = Vec::new();

                for (monster_idx, data) in payload.m {
                    let resolved_name = self.identity_mapper.resolve_actor(&format!("m:{}", monster_idx));
                    if data.hp.is_some() || data.shield.is_some() {
                        stats_deltas.push(StatTimelineDelta {
                            target: resolved_name,
                            hp: data.hp,
                            mp: None,
                            shield: data.shield,
                        });
                    }
                }

                for (username, data) in payload.p {
                    if data.hp.is_some() || data.mp.is_some() {
                        stats_deltas.push(StatTimelineDelta {
                            target: username,
                            hp: data.hp,
                            mp: data.mp,
                            shield: None,
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
                    let action_type = match aura_event.cmd.as_str() {
                        "aura+" => "Applied",
                        "aura++" => "Refreshed",
                        "aura-" => "Faded",
                        other => other,
                    }.to_string();
            
                    
                    let caster_name = aura_event.caster
                        .map(|c| self.identity_mapper.resolve_actor(&c))
                        .unwrap_or_else(|| "System".to_string());

     
            
                    if let Some(details) = aura_event.aura {
                        aura_deltas.push(AuraTimelineDelta {
                            action: action_type.clone(),
                            caster: caster_name.clone(),
                            target: self.identity_mapper.resolve_actor(&aura_event.target),
                            aura_name: details.name,
                            value: details.val,
                            duration: details.duration,
                            aura_type: details.aura_type,
                            is_new: details.is_new,
                        });
                    }
            
                    if let Some(bulk_details) = aura_event.auras {
                        for details in bulk_details {
                            aura_deltas.push(AuraTimelineDelta {
                                action: action_type.clone(),
                                caster: caster_name.clone(),
                                target: self.identity_mapper.resolve_actor(&aura_event.target),
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