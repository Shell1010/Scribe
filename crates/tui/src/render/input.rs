use crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use crate::app::{App, DragSide, SearchTarget};

use super::clipboard::copy_to_clipboard;

pub fn handle_event(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event: Event,
) -> bool {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) => {
            let _ = handle_mouse(terminal, app, mouse);
        }
        _ => {}
    }
    false
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    if app.show_help {
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
        ) {
            app.show_help = false;
        }
        return;
    }

    if app.search_mode {
        match key.code {
            KeyCode::Esc => {
                app.search_mode = false;
                app.input_buffer.clear();
            }
            KeyCode::Enter => {
                app.search_query = app.input_buffer.clone();
                app.recalc_search();
                if !app.search_matches.is_empty() {
                    app.clamp_search_match();
                    app.selected_log_index = app.search_matches[app.search_match_idx];
                }
                app.search_mode = false;
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        }
        return;
    }

    if app.filter_mode {
        handle_filter_key(app, key);
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,

        KeyCode::Char('?') => {
            app.show_help = true;
        }

        KeyCode::Char('/') if app.active_tab == 0 => {
            app.search_mode = true;
            app.input_buffer.clear();
        }

        KeyCode::Char('/') if app.active_tab == 4 => {
            app.filter_mode = true;
            app.filter_target = SearchTarget::PartyFilter;
            app.input_buffer.clear();
        }

        KeyCode::Char('/') if app.active_tab == 1 => {
            app.filter_mode = true;
            app.filter_target = SearchTarget::SkillFilter;
            app.input_buffer.clear();
        }

        KeyCode::Char('/') if app.active_tab == 2 => {
            app.filter_mode = true;
            app.filter_target = SearchTarget::DropFilter;
            app.input_buffer.clear();
        }

        KeyCode::Char('n') if !app.search_query.is_empty() && app.active_tab == 0 => {
            app.jump_to_match(true);
            app.set_status(format!(
                "Match {}/{}",
                app.search_match_idx + 1,
                app.search_matches.len()
            ));
        }

        KeyCode::Char('N') if !app.search_query.is_empty() && app.active_tab == 0 => {
            app.jump_to_match(false);
            app.set_status(format!(
                "Match {}/{}",
                app.search_match_idx + 1,
                app.search_matches.len()
            ));
        }

        KeyCode::Char(' ') => {
            app.paused = !app.paused;
            app.set_status(if app.paused {
                "PAUSED".into()
            } else {
                "Resumed".into()
            });
        }

        KeyCode::Enter => {
            app.active_tab = (app.active_tab + 1) % app.tabs.len();
            app.snap = true;
            app.highlighted_entity = None;
        }

        KeyCode::Char('1') if !app.search_mode => {
            app.active_tab = 0;
            app.snap = true;
            app.highlighted_entity = None;
        }
        KeyCode::Char('2') if !app.search_mode => {
            app.active_tab = 1;
            app.highlighted_entity = None;
        }
        KeyCode::Char('3') if !app.search_mode => {
            app.active_tab = 2;
            app.highlighted_entity = None;
        }
        KeyCode::Char('4') if !app.search_mode => {
            app.active_tab = 3;
            app.highlighted_entity = None;
        }
        KeyCode::Char('5') if !app.search_mode => {
            app.active_tab = 4;
            app.highlighted_entity = None;
        }

        KeyCode::BackTab => {
            app.active_tab = if app.active_tab == 0 {
                app.tabs.len() - 1
            } else {
                app.active_tab - 1
            };
            app.snap = app.active_tab == 0;
            app.highlighted_entity = None;
        }

        KeyCode::Up => handle_up(app),
        KeyCode::Down => handle_down(app),

        KeyCode::Left => match app.active_tab {
            0 => {
                app.scroll_log_x = app.scroll_log_x.saturating_sub(2);
            }
            1 => {
                app.scroll_class_x = app.scroll_class_x.saturating_sub(2);
            }
            3 => app.mechanics.cycle_strategy_backward(),
            _ => {}
        },
        KeyCode::Right => match app.active_tab {
            0 => {
                app.scroll_log_x = app.scroll_log_x.saturating_add(2);
            }
            1 => {
                app.scroll_class_x = app.scroll_class_x.saturating_add(2);
            }
            3 => app.mechanics.cycle_strategy_forward(),
            _ => {}
        },

        KeyCode::PageUp => {
            if app.active_tab == 0 {
                app.selected_log_index = app.selected_log_index.saturating_sub(20);
                app.snap = false;
            }
        }
        KeyCode::PageDown => {
            if app.active_tab == 0 {
                app.selected_log_index =
                    (app.selected_log_index + 20).min(app.system_log.len().saturating_sub(1));
                app.snap = false;
            }
        }

        KeyCode::Char('c') => {
            let copied = match app.active_tab {
                0 => app
                    .system_log
                    .get(app.selected_log_index)
                    .map(|s| s.as_str().to_owned()),
                1 => {
                    let all_skills = collect_skills(app);
                    all_skills
                        .get(app.selected_skill_index)
                        .map(|(skill, _)| serde_json::to_string_pretty(skill).unwrap_or_default())
                }
                2 => {
                    let drops = collect_drops(app);
                    drops.get(app.scroll_metrics_y as usize).map(|d| {
                        format!(
                            "{} | Qty: {} | Rate: {:.2}% | 1 in {} kills",
                            d.name,
                            d.total_quantity,
                            d.drop_count as f64 / app.combat_metrics.total_kills.max(1) as f64
                                * 100.0,
                            (1.0 / (d.drop_count as f64
                                / app.combat_metrics.total_kills.max(1) as f64))
                                .round() as i32
                        )
                    })
                }
                4 => {
                    let entities = collect_entities(app);
                    entities.get(app.selected_entity_index).map(|(_, e)| {
                        if e.is_monster {
                            format!(
                                "{} HP:{}/{} SG:{} Auras:{}",
                                e.name,
                                e.hp,
                                e.max_hp,
                                e.shield,
                                e.auras.join(", ")
                            )
                        } else {
                            format!(
                                "{} Lvl {} [{}] HP:{}/{} MP:{}/{}",
                                e.name, e.level, e.class, e.hp, e.max_hp, e.mp, e.max_mp
                            )
                        }
                    })
                }
                _ => None,
            };
            if let Some(text) = copied {
                if !text.is_empty() && copy_to_clipboard(&text) {
                    let preview = if text.len() > 40 { &text[..40] } else { &text };
                    app.set_status(format!("Copied: {}...", preview));
                } else {
                    app.set_status("Copy failed (no clipboard tool found)".into());
                }
            }
        }
        KeyCode::Char('d') => {
            if app.active_tab == 2 {
                app.show_dps_graph = !app.show_dps_graph;
                app.set_status(if app.show_dps_graph {
                    "DPS Graphs".into()
                } else {
                    "Drop Logistics".into()
                });
            }
        }
        KeyCode::Char('G') => {
            if app.active_tab == 0 {
                app.selected_log_index = app.system_log.len().saturating_sub(1);
            }
        }
        KeyCode::Char('g') if app.active_tab == 0 => {
            app.selected_log_index = 0;
        }
        _ => {}
    }
}

fn handle_filter_key(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.filter_mode = false;
            app.input_buffer.clear();
            match app.filter_target {
                SearchTarget::PartyFilter => app.party_filter.clear(),
                SearchTarget::SkillFilter => app.skill_filter.clear(),
                SearchTarget::DropFilter => app.drop_filter.clear(),
                _ => {}
            }
        }
        KeyCode::Enter => {
            match app.filter_target {
                SearchTarget::PartyFilter => app.party_filter = app.input_buffer.clone(),
                SearchTarget::SkillFilter => app.skill_filter = app.input_buffer.clone(),
                SearchTarget::DropFilter => app.drop_filter = app.input_buffer.clone(),
                _ => {}
            }
            app.filter_mode = false;
            app.input_buffer.clear();
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        _ => {}
    }
}

fn collect_skills<'a>(
    app: &'a App,
) -> Vec<(&'a serde_json::Map<String, serde_json::Value>, &'a str)> {
    let q = app.skill_filter.to_lowercase();
    let mut all = Vec::new();
    for s in &app.class_info.active_skills {
        all.push((s, "Active"));
    }
    for s in &app.class_info.passive_skills {
        all.push((s, "Passive"));
    }
    if q.is_empty() {
        return all;
    }
    all.into_iter()
        .filter(|(skill, _)| {
            skill
                .get("nam")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase()
                .contains(&q)
        })
        .collect()
}

fn collect_drops(app: &App) -> Vec<&crate::app::DropMetric> {
    let q = app.drop_filter.to_lowercase();
    let mut drops: Vec<_> = app.combat_metrics.drops.values().collect();
    drops.sort_by_key(|a| std::cmp::Reverse(a.drop_count));
    if q.is_empty() {
        return drops;
    }
    drops
        .into_iter()
        .filter(|d| d.name.to_lowercase().contains(&q))
        .collect()
}

pub fn collect_entities<'a>(app: &'a App) -> Vec<(&'a String, &'a crate::app::EntityState)> {
    let q = app.party_filter.to_lowercase();
    let mut players: Vec<_> = app.entities.iter().filter(|(_, e)| !e.is_monster).collect();
    players.sort_by(|a, b| a.0.cmp(b.0));
    let mut monsters: Vec<_> = app.entities.iter().filter(|(_, e)| e.is_monster).collect();
    monsters.sort_by_key(|a| std::cmp::Reverse(a.1.hp));
    let mut entities = players;
    entities.extend(monsters);
    if q.is_empty() {
        return entities;
    }
    entities
        .into_iter()
        .filter(|(name, _)| name.to_lowercase().contains(&q))
        .collect()
}

fn handle_up(app: &mut App) {
    match app.active_tab {
        0 => {
            app.selected_log_index = app.selected_log_index.saturating_sub(1);
            app.snap = false;
        }
        1 => {
            app.selected_skill_index = app.selected_skill_index.saturating_sub(1);
        }
        2 => {
            app.scroll_metrics_y = app.scroll_metrics_y.saturating_sub(1);
        }
        3 => app.mechanics.cycle_role_backward(),
        4 => {
            app.selected_entity_index = app.selected_entity_index.saturating_sub(1);
        }
        _ => {}
    }
}

fn handle_down(app: &mut App) {
    match app.active_tab {
        0 => {
            if app.selected_log_index + 1 < app.system_log.len() {
                app.selected_log_index += 1;
            }
            app.snap = false;
        }
        1 => {
            app.selected_skill_index = app.selected_skill_index.saturating_add(1);
        }
        2 => {
            app.scroll_metrics_y = app.scroll_metrics_y.saturating_add(1);
        }
        3 => app.mechanics.cycle_role_forward(),
        4 => {
            let total = collect_entities(app).len().saturating_sub(1);
            if app.selected_entity_index < total {
                app.selected_entity_index += 1;
            }
        }
        _ => {}
    }
}

fn handle_mouse(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mouse: crossterm::event::MouseEvent,
) -> io::Result<()> {
    let row = mouse.row;
    let col = mouse.column;
    let term_h = terminal.size()?.height;
    let term_w = terminal.size()?.width;

    let menu_w = term_w / 6;
    let content_x = menu_w;
    let main_h = term_h.saturating_sub(1);

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if row >= main_h {
                return Ok(());
            }
            if col >= content_x.saturating_sub(1) && col <= content_x {
                app.dragging = Some(DragSide::Left);
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if let Some(side) = app.dragging {
                match side {
                    DragSide::Left => {
                        app.split_left = col.max(12);
                    }
                }
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            app.dragging = None;
        }
        MouseEventKind::ScrollUp => match app.active_tab {
            0 => {
                app.selected_log_index = app.selected_log_index.saturating_sub(3);
                app.snap = false;
            }
            1 => {
                app.selected_skill_index = app.selected_skill_index.saturating_sub(3);
            }
            2 => {
                app.scroll_metrics_y = app.scroll_metrics_y.saturating_sub(3);
            }
            4 => {
                app.selected_entity_index = app.selected_entity_index.saturating_sub(3);
            }
            _ => {}
        },
        MouseEventKind::ScrollDown => match app.active_tab {
            0 => {
                app.selected_log_index =
                    (app.selected_log_index + 3).min(app.system_log.len().saturating_sub(1));
                app.snap = false;
            }
            1 => {
                let total = collect_skills(app).len().saturating_sub(1);
                app.selected_skill_index = (app.selected_skill_index + 3).min(total);
            }
            2 => {
                app.scroll_metrics_y = app.scroll_metrics_y.saturating_add(3);
            }
            4 => {
                let total = collect_entities(app).len().saturating_sub(1);
                app.selected_entity_index = (app.selected_entity_index + 3).min(total);
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}
