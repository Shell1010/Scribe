use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
    Frame,
};
use std::collections::HashSet;
use std::time::Duration;

use super::sparkline::render_sparkline;
use crate::app::{App, EntityState};

pub fn render_output_log(f: &mut Frame, app: &mut App, area: Rect) {
    app.clamp_log_selection();

    if app.snap {
        if !app.system_log.is_empty() {
            app.selected_log_index = app.system_log.len() - 1;
        }
        app.snap = false;
    }

    let vis = area.width.saturating_sub(2) as usize;
    if let Some(max_len) = app.system_log.iter().map(|s| s.chars().count()).max() {
        let max_scroll = (max_len as u16).saturating_sub(vis as u16);
        app.scroll_log_x = app.scroll_log_x.min(max_scroll);
    }
    let skip = app.scroll_log_x as usize;

    let search_set: HashSet<usize> = app.search_matches.iter().copied().collect();

    let log_items: Vec<ListItem> = app
        .system_log
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let is_selected = i == app.selected_log_index;
            let is_match = search_set.contains(&i);

            let start = s.char_indices().nth(skip).map(|(i, _)| i).unwrap_or(s.len());
            let remaining = &s[start..];
            let end = remaining.char_indices().nth(vis).map(|(i, _)| i).unwrap_or(remaining.len());
            let visible = &s[start..start + end];

            let spans = if app.search_query.is_empty() {
                vec![Span::styled(
                    visible,
                    if is_selected {
                        Style::default().fg(Color::Black).bg(Color::White)
                    } else {
                        Style::default().fg(Color::White)
                    },
                )]
            } else {
                super::highlight_search(visible, &app.search_query, is_match)
                    .into_iter()
                    .map(|span| {
                        if is_selected && span.style == Style::default() {
                            Span::styled(
                                span.content,
                                Style::default().fg(Color::Black).bg(Color::White),
                            )
                        } else {
                            span
                        }
                    })
                    .collect()
            };

            ListItem::new(Line::from(spans))
        })
        .collect();

    let line = app.selected_log_index + 1;
    let total = app.system_log.len();
    let paused_tag = if app.paused { " [PAUSED]" } else { "" };
    let search_tag = if app.search_mode {
        format!(" | /{}", app.input_buffer)
    } else if !app.search_query.is_empty() {
        let total_m = app.search_matches.len();
        let cur_m = if total_m > 0 {
            app.search_match_idx + 1
        } else {
            0
        };
        format!(" | /{} ({}/{})", app.search_query, cur_m, total_m)
    } else {
        String::new()
    };

    let log_list = List::new(log_items).block(
        Block::default()
            .title(format!(
                " Live {} / {}{}{} ",
                line, total, paused_tag, search_tag
            ))
            .borders(Borders::ALL),
    );

    let mut list_state = ListState::default();
    if !app.system_log.is_empty() {
        list_state.select(Some(app.selected_log_index));
    }
    f.render_stateful_widget(log_list, area, &mut list_state);
}

pub fn render_class_data(f: &mut Frame, app: &mut App, area: Rect) {
    let class_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    let mrm_text = app.class_info.mrm.join(" ");
    let header_text = format!(
        "Name: {} | Category: {}\nMRM: {}\n\nDesc: {}",
        app.class_info.name, app.class_info.category, mrm_text, app.class_info.desc
    );
    let header = Paragraph::new(header_text).wrap(Wrap { trim: true }).block(
        Block::default()
            .title(" Class Profile ")
            .borders(Borders::ALL),
    );
    f.render_widget(header, class_chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(class_chunks[1]);

    let q = app.skill_filter.to_lowercase();
    let skill_count = if q.is_empty() {
        app.class_info.active_skills.len() + app.class_info.passive_skills.len()
    } else {
        app.class_info
            .active_skills
            .iter()
            .chain(app.class_info.passive_skills.iter())
            .filter(|s| {
                s.get("nam")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
            })
            .count()
    };
    app.selected_skill_index = app.selected_skill_index.min(skill_count.saturating_sub(1));
    let filtered_skills = collect_filtered_skills(app);

    let filter_tag = if app.filter_mode && app.filter_target == crate::app::SearchTarget::SkillFilter {
        format!(" [filter: {}]", app.input_buffer)
    } else if !app.skill_filter.is_empty() {
        format!(" [filter: {}]", app.skill_filter)
    } else {
        String::new()
    };

    let mut lines: Vec<ListItem> = Vec::new();
    let mut last_type = "";
    for (i, (skill, s_type)) in filtered_skills.iter().enumerate() {
        if *s_type != last_type && !last_type.is_empty() {
            lines.push(ListItem::new(Line::from(Span::styled(
                "─────────────────────────",
                Style::default().fg(Color::DarkGray),
            ))));
        }
        last_type = s_type;

        let name = skill
            .get("nam")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let typ = skill.get("typ").and_then(|v| v.as_str()).unwrap_or("-");
        let dsrc = skill.get("dsrc").and_then(|v| v.as_str()).unwrap_or("-");
        let damage = skill.get("damage").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let display = format!(
            "[{}] {} (typ: {}, tgt: {}, dmg: {})",
            s_type, name, typ, dsrc, damage
        );

        let style = if i == app.selected_skill_index {
            Style::default().fg(Color::Black).bg(Color::White)
        } else if s_type == &"Active" {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(ListItem::new(Line::from(Span::styled(display, style))));
    }

    let skill_list = List::new(lines).block(
        Block::default()
            .title(format!(" Skills (Up/Down){} ", filter_tag))
            .borders(Borders::ALL),
    );

    let mut list_state = ListState::default();
    if !filtered_skills.is_empty() {
        list_state.select(Some(app.selected_skill_index));
    }
    f.render_stateful_widget(skill_list, body_chunks[0], &mut list_state);

    let mut details_text = String::new();
    if let Some(&(selected_skill, _)) = filtered_skills.get(app.selected_skill_index) {
        if let Some(n) = selected_skill.get("nam") {
            details_text.push_str(&format!("NAME: {}\n\n", n.as_str().unwrap_or("")));
        }

        details_text.push_str("--- RAW PROPERTIES ---\n");
        for (k, v) in selected_skill {
            if k != "nam" && k != "desc" && k != "auras" {
                let pretty_val =
                    serde_json::to_string_pretty(v).unwrap_or_else(|_| format!("{}", v));
                details_text.push_str(&format!("{}: {}\n", k, pretty_val));
            }
        }
    }

    let detail_panel = Paragraph::new(details_text)
        .scroll((app.scroll_class_y, app.scroll_class_x))
        .block(
            Block::default()
                .title(" Verbose Data ")
                .borders(Borders::ALL),
        );
    f.render_widget(detail_panel, body_chunks[1]);
}

fn collect_filtered_skills<'a>(app: &'a App) -> Vec<(&'a serde_json::Map<String, serde_json::Value>, &'a str)> {
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

pub fn render_combat_metrics(f: &mut Frame, app: &mut App, area: Rect) {
    let metrics_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    let elapsed = app.combat_metrics.session_start.elapsed().as_secs_f64() / 60.0;
    let mins = elapsed.max(1.0);

    let recent_group_dps = app.combat_metrics.dps.recent_group_dps();
    let total_group_dps = app.combat_metrics.dps.total_group_dps();
    let recent_solo_dps = app.combat_metrics.dps.recent_solo_dps();
    let total_solo_dps = app.combat_metrics.dps.total_solo_dps();
    let kills = app.combat_metrics.total_kills;
    let gold = app.combat_metrics.total_gold;
    let exp = app.combat_metrics.total_exp;

    let header_text = format!(
        "Session Time: {:.1} Minutes\n\nTotal Kills: {} ({:.1} KPM)\nTotal Gold: {} ({}/min)\nTotal Exp: {} ({}/min)\nGroup DPS: {} (Recent: {})\nSolo DPS: {} (Recent: {})",
        elapsed,
        kills, kills as f64 / mins,
        App::format_compact(gold as f64), App::format_compact(gold as f64 / mins),
        App::format_compact(exp as f64), App::format_compact(exp as f64 / mins),
        App::format_compact(total_group_dps), App::format_compact(recent_group_dps),
        App::format_compact(total_solo_dps), App::format_compact(recent_solo_dps),
    );

    let header = Paragraph::new(header_text).block(
        Block::default()
            .title(" Global Session Stats ")
            .borders(Borders::ALL),
    );
    f.render_widget(header, metrics_chunks[0]);

    if app.show_dps_graph {
        render_dps_graphs(f, app, metrics_chunks[1]);
    } else {
        render_drop_logistics(f, app, metrics_chunks[1]);
    }
}

fn render_dps_graphs(f: &mut Frame, app: &App, area: Rect) {
    let graph_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(graph_chunks[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(graph_chunks[1]);

    let peak_age = app.dps_peak_at.elapsed().as_secs_f64();
    let stale = peak_age > 30.0;

    let y_max_from = |extract: fn(&crate::app::DpsSample) -> f64| -> f64 {
        if stale {
            app.dps_history
                .iter()
                .map(extract)
                .fold(0.0_f64, f64::max)
                .max(1.0)
        } else {
            app.dps_peak
        }
    };

    render_sparkline(
        f,
        left[0],
        &app.dps_history,
        |s| s.group,
        "Group DPS",
        ratatui::style::Color::Green,
        y_max_from(|s| s.group),
    );
    render_sparkline(
        f,
        left[1],
        &app.dps_history,
        |s| s.recent_group,
        "Recent GDPS",
        ratatui::style::Color::Cyan,
        y_max_from(|s| s.recent_group),
    );
    render_sparkline(
        f,
        right[0],
        &app.dps_history,
        |s| s.solo,
        "Solo DPS",
        ratatui::style::Color::Yellow,
        y_max_from(|s| s.solo),
    );
    render_sparkline(
        f,
        right[1],
        &app.dps_history,
        |s| s.recent_solo,
        "Recent SDPS",
        ratatui::style::Color::Magenta,
        y_max_from(|s| s.recent_solo),
    );
}

fn render_drop_logistics(f: &mut Frame, app: &mut App, area: Rect) {
    let q = app.drop_filter.to_lowercase();
    let mut drops: Vec<&crate::app::DropMetric> = app.combat_metrics.drops.values().collect();
    drops.sort_by_key(|a| std::cmp::Reverse(a.drop_count));
    if !q.is_empty() {
        drops.retain(|d| d.name.to_lowercase().contains(&q));
    }

    let filter_tag = if app.filter_mode && app.filter_target == crate::app::SearchTarget::DropFilter {
        format!(" [filter: {}]", app.input_buffer)
    } else if !app.drop_filter.is_empty() {
        format!(" [filter: {}]", app.drop_filter)
    } else {
        String::new()
    };

    let header_row = Row::new(vec![
        "Item Name",
        "Total Qty",
        "Drop Rate",
        "1 in X Kills",
        "Since Last",
        "Next Est.",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let current_kills = app.combat_metrics.total_kills;

    let rows: Vec<Row> = drops
        .iter()
        .map(|d| {
            let empirical_rate = d.drop_count as f64 / current_kills.max(1) as f64;
            let one_in_x = (1.0 / empirical_rate).round() as i32;
            let kills_since_last = current_kills.saturating_sub(d.kills_at_last_drop);

            let next_est = one_in_x - kills_since_last as i32;
            let next_str = if next_est <= 0 {
                format!("Overdue ({})", next_est.abs())
            } else {
                format!("in {}", next_est)
            };

            Row::new(vec![
                d.name.clone(),
                d.total_quantity.to_string(),
                format!("{:.2}%", empirical_rate * 100.0),
                one_in_x.to_string(),
                kills_since_last.to_string(),
                next_str,
            ])
            .style(Style::default().fg(if next_est <= 0 {
                Color::Green
            } else {
                Color::White
            }))
        })
        .collect();

    let max_scroll = rows
        .len()
        .saturating_sub(area.height.saturating_sub(3) as usize) as u16;
    app.scroll_metrics_y = app.scroll_metrics_y.min(max_scroll);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(format!(" Drop Logistics (Up/Down){} ", filter_tag))
            .borders(Borders::ALL),
    );

    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(app.scroll_metrics_y as usize));
    f.render_stateful_widget(table, area, &mut table_state);
}

pub fn render_ultra_mechanics(f: &mut Frame, app: &mut App, area: Rect) {
    let strategies = &app.mechanics.active_boss;
    if strategies.is_empty() {
        let paragraph =
            Paragraph::new("No active boss mechanics loaded. Waiting for configuration...").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Boss Mechanics "),
            );
        f.render_widget(paragraph, area);
        return;
    }

    let mechanic_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let mut items = vec![];
    if let Some(strategy) = strategies.get(app.mechanics.strategy_index) {
        items.push(Line::from(vec![Span::styled(
            "Active Ultra",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        items.push(Line::from(format!(
            "Selected Strategy (Left/Right): {}",
            strategy.name
        )));
        items.push(Line::from(format!(
            "Current Phase: {}",
            app.mechanics.current_phase
        )));
        items.push(Line::from(""));
        items.push(Line::from("Select Your Role (Up/Down):"));

        for role in &strategy.roles {
            let is_active = *role == app.mechanics.selected_role || *role == "all";
            let style = if is_active {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let prefix = if is_active { "[X]" } else { "[ ]" };
            items.push(Line::from(Span::styled(
                format!("  {} {}", prefix, role),
                style,
            )));
        }
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Boss Configuration "),
    );
    f.render_widget(list, mechanic_chunks[0]);

    if let Some((action, timestamp)) = &app.mechanics.active_alert
        && timestamp.elapsed() < Duration::from_secs(2) {
            let remaining = 2.0 - timestamp.elapsed().as_secs_f64();
            let (alert_text, bg_color, fg_color) = match action {
                scribe_core::mechanics::Action::Taunt => (
                    format!("\n\n!!! TAUNT NOW !!!\n!!! TAUNT NOW !!!\n!!! TAUNT NOW !!!\n\n  [{:.1}s]", remaining),
                    Color::Blue,
                    Color::White,
                ),
                scribe_core::mechanics::Action::Zone => (
                    format!("\n\n!!! ZONE NOW !!!\n!!! ZONE NOW !!!\n!!! ZONE NOW !!!\n\n  [{:.1}s]", remaining),
                    Color::Red,
                    Color::White,
                ),
                scribe_core::mechanics::Action::Decay => (
                    format!("\n\n!!! DECAY NOW !!!\n!!! DECAY NOW !!!\n!!! DECAY NOW !!!\n\n  [{:.1}s]", remaining),
                    Color::Green,
                    Color::White,
                ),
                scribe_core::mechanics::Action::Quixotic => (
                    format!("\n\n!!! QUIX NOW !!!\n!!! QUIX NOW !!!\n!!! QUIX NOW !!!\n\n  [{:.1}s]", remaining),
                    Color::Yellow,
                    Color::Black,
                ),
            };

            let alert_paragraph = Paragraph::new(alert_text)
                .block(Block::default().borders(Borders::ALL))
                .style(
                    Style::default()
                        .fg(fg_color)
                        .bg(bg_color)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(ratatui::layout::Alignment::Center);

            f.render_widget(alert_paragraph, mechanic_chunks[1]);
        } else if let Some((action, timestamp)) = &app.last_alert
        && timestamp.elapsed() < Duration::from_secs(3) {
            let (text, color) = match action {
                scribe_core::mechanics::Action::Taunt => ("TAUNT", Color::Blue),
                scribe_core::mechanics::Action::Zone => ("ZONE", Color::Red),
                scribe_core::mechanics::Action::Decay => ("DECAY", Color::Green),
                scribe_core::mechanics::Action::Quixotic => ("QUIXOTIC", Color::Yellow),
            };
            let faded = Paragraph::new(format!("\n\n\n  Last: {}", text))
                .block(Block::default().borders(Borders::ALL).title(" Alerts "))
                .style(
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::ITALIC),
                )
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(faded, mechanic_chunks[1]);
        }
}

pub fn render_party_status(f: &mut Frame, app: &mut App, area: Rect) {
    let filter_tag = if app.filter_mode && app.filter_target == crate::app::SearchTarget::PartyFilter {
        format!(" [filter: {}]", app.input_buffer)
    } else if !app.party_filter.is_empty() {
        format!(" [filter: {}]", app.party_filter)
    } else {
        String::new()
    };

    let q = app.party_filter.to_lowercase();

    let mut players: Vec<_> = app
        .entities
        .iter()
        .filter(|(_, e)| !e.is_monster)
        .filter(|(name, _)| q.is_empty() || name.to_lowercase().contains(&q))
        .collect();
    players.sort_by(|a, b| a.0.cmp(b.0));

    let mut monsters: Vec<_> = app
        .entities
        .iter()
        .filter(|(_, e)| e.is_monster)
        .filter(|(name, _)| q.is_empty() || name.to_lowercase().contains(&q))
        .collect();
    monsters.sort_by_key(|a| std::cmp::Reverse(a.1.hp));

    let total_visible = players.len() + monsters.len();
    if total_visible == 0 {
        app.selected_entity_index = 0;
    } else {
        app.selected_entity_index = app.selected_entity_index.min(total_visible - 1);
    }

    if let Some(ref highlighted) = app.highlighted_entity {
        if let Some(idx) = players
            .iter()
            .chain(monsters.iter())
            .position(|(name, _)| name.as_str() == highlighted.as_str())
        {
            app.selected_entity_index = idx;
        }
        app.highlighted_entity = None;
    }

    let party_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_entity_list(
        f,
        party_chunks[0],
        &players,
        &format!(" Players ({}){} ", players.len(), filter_tag),
        0,
        app.selected_entity_index,
        &app.active_monsters,
        &app.username,
    );
    draw_entity_list(
        f,
        party_chunks[1],
        &monsters,
        &format!(" Monsters ({}) ", monsters.len()),
        players.len(),
        app.selected_entity_index,
        &app.active_monsters,
        &app.username,
    );
}

fn draw_entity_list(
    f: &mut Frame,
    area: Rect,
    entities: &[(&String, &EntityState)],
    title: &str,
    offset: usize,
    selected_idx: usize,
    active_monsters: &std::collections::HashMap<String, crate::app::MonsterCombatState>,
    username: &str,
) {
    let mut lines: Vec<ListItem> = Vec::new();
    for (i, (name, ent)) in entities.iter().enumerate() {
        let global_idx = offset + i;
        let is_sel = global_idx == selected_idx;
        let hp_pct = if ent.max_hp > 0 {
            ent.hp as f64 / ent.max_hp as f64
        } else {
            0.0
        };
        let mp_pct = if ent.max_mp > 0 {
            ent.mp as f64 / ent.max_mp as f64
        } else {
            0.0
        };

        let hp_bar_w = 20;
        let hp_color = if hp_pct > 0.6 {
            Color::Green
        } else if hp_pct > 0.3 {
            Color::Yellow
        } else {
            Color::Red
        };

        let filled = (hp_pct * hp_bar_w as f64) as usize;
        let hp_bar = format!("{}{}", "#".repeat(filled), "-".repeat(hp_bar_w - filled));

        let name_color = if is_sel { Color::Black } else { Color::White };
        let name_bg = if is_sel { Color::White } else { Color::Reset };

        let header = if ent.is_monster {
            format!(
                "{} (HP: {}/{} | SG: {})",
                name, ent.hp, ent.max_hp, ent.shield
            )
        } else if ent.max_hp > 0 || ent.max_mp > 0 {
            format!("{} Lvl {} [{}]", name, ent.level, ent.class)
        } else {
            name.to_string()
        };

        lines.push(ListItem::new(Line::from(Span::styled(
            header,
            Style::default()
                .fg(name_color)
                .bg(name_bg)
                .add_modifier(Modifier::BOLD),
        ))));

        let hp_line = format!("  HP [{}] {:.0}%", hp_bar, hp_pct * 100.0);
        lines.push(ListItem::new(Line::from(Span::styled(
            hp_line,
            Style::default().fg(hp_color),
        ))));

        if ent.shield > 0 && !ent.is_monster {
            lines.push(ListItem::new(Line::from(Span::styled(
                format!("  Shield: {}", ent.shield),
                Style::default().fg(Color::Magenta),
            ))));
        }

        if ent.max_mp > 0 {
            let mp_bar_w = 20;
            let mp_filled = (mp_pct * mp_bar_w as f64) as usize;
            let mp_bar: String = "#".repeat(mp_filled) + &"-".repeat(mp_bar_w - mp_filled);
            let mp_line = format!("  MP [{}] {:.0}%", mp_bar, mp_pct * 100.0);
            lines.push(ListItem::new(Line::from(Span::styled(
                mp_line,
                Style::default().fg(Color::Cyan),
            ))));
        }

        if !ent.auras.is_empty() {
            lines.push(ListItem::new(Line::from(Span::styled(
                format!("  Auras: {}", ent.auras.join(", ")),
                Style::default().fg(Color::DarkGray),
            ))));
        }

        if ent.is_monster {
            if let Some(state) = active_monsters.get(name.as_str()) {
                if state.damage_dealt > 0 {
                    lines.push(ListItem::new(Line::from(Span::styled(
                        format!("  Your DMG: {}", state.damage_dealt),
                        Style::default().fg(Color::Cyan),
                    ))));
                }
            }
        } else if name.as_str() == username {
            lines.push(ListItem::new(Line::from(Span::styled(
                "  (you)",
                Style::default().fg(Color::DarkGray),
            ))));
        }

        lines.push(ListItem::new(Line::from("")));
    }

    let list = List::new(lines).block(
        Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL),
    );
    f.render_widget(list, area);
}
