use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap, Row, Table},
    Terminal, Frame,
};
use std::{io, time::Duration};
use std::sync::mpsc::Receiver;
use scribe_parser::ScribeParser; 


use crate::app::{App, DropMetric}; 
pub mod app;

pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(15), Constraint::Percentage(70), Constraint::Percentage(15)].as_ref())
        .split(f.area());


    let tab_items: Vec<ListItem> = app.tabs.iter().enumerate().map(|(i, &tab_name)| {
        let style = if i == app.active_tab {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(Line::from(Span::styled(format!(" {} ", tab_name), style)))
    }).collect();

    let tabs_list = List::new(tab_items).block(
        Block::default().title(" Menu (Enter) ").borders(Borders::ALL)
    );
    f.render_widget(tabs_list, chunks[0]);


    match app.active_tab {
        0 => {
            let log_text = app.system_log.iter().cloned().collect::<Vec<String>>().join("\n");
            let log_height = chunks[1].height.saturating_sub(2);
            let total_lines = log_text.lines().count() as u16;
            let max_scroll = total_lines.saturating_sub(log_height);

            if app.snap {
                app.scroll_output_y = max_scroll;
                app.snap = false;
            } else {
                app.scroll_output_y = app.scroll_output_y.min(max_scroll);
            }
        
            let log_paragraph = Paragraph::new(log_text)
                .scroll((app.scroll_output_y, app.scroll_output_x))
                .block(Block::default().title(" Live Output ").borders(Borders::ALL));

            f.render_widget(log_paragraph, chunks[1]);
        }
        1 => {
            let class_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(8), Constraint::Min(0)]) // Header, Body
                .split(chunks[1]);

            let mrm_text = app.class_info.mrm.join(" ");
            let header_text = format!(
                "Name: {} | Category: {}\nMRM: {}\n\nDesc: {}",
                app.class_info.name, app.class_info.category, mrm_text, app.class_info.desc
            );
            let header = Paragraph::new(header_text)
                .wrap(Wrap { trim: true })
                .block(Block::default().title(" Class Profile ").borders(Borders::ALL));
            f.render_widget(header, class_chunks[0]);

            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(class_chunks[1]);

            let mut all_skills = Vec::new();
            all_skills.extend(app.class_info.active_skills.iter().map(|s| (s, "Active")));
            all_skills.extend(app.class_info.passive_skills.iter().map(|s| (s, "Passive")));

            // bound the selection index just in case
            let max_index = all_skills.len().saturating_sub(1);
            app.selected_skill_index = app.selected_skill_index.min(max_index);

            let list_items: Vec<ListItem> = all_skills.iter().enumerate().map(|(i, (skill, s_type))| {
                let name = skill.get("nam").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let typ = skill.get("typ").and_then(|v| v.as_str()).unwrap_or("-");
                let dsrc = skill.get("dsrc").and_then(|v| v.as_str()).unwrap_or("-");
                let damage = skill.get("damage").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                let display = format!("[{}] {} (typ: {}, tgt: {}, dmg: {})", s_type, name, typ, dsrc, damage);
                
                let style = if i == app.selected_skill_index {
                    Style::default().fg(Color::Black).bg(Color::White) // Highlight selected
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(Span::styled(display, style)))
            }).collect();

            let skill_list = List::new(list_items)
                .block(Block::default().title(" Skills (Up/Down) ").borders(Borders::ALL));
            
            let mut list_state = ListState::default();
            if !all_skills.is_empty() { list_state.select(Some(app.selected_skill_index)); }
            f.render_stateful_widget(skill_list, body_chunks[0], &mut list_state);

            let mut details_text = String::new();
            if let Some(&(selected_skill, _)) = all_skills.get(app.selected_skill_index) {
                if let Some(n) = selected_skill.get("nam") { details_text.push_str(&format!("NAME: {}\n\n", n.as_str().unwrap_or(""))); }
                
                details_text.push_str("--- RAW PROPERTIES ---\n");
                for (k, v) in selected_skill {
                    if k != "nam" && k != "desc" && k != "auras" {
                        let pretty_val = serde_json::to_string_pretty(v).unwrap_or_else(|_| format!("{}", v));
                        details_text.push_str(&format!("{}: {}\n", k, pretty_val));
                    }
                }
            }

            let detail_panel = Paragraph::new(details_text)
                .scroll((app.scroll_class_y, app.scroll_class_x))
                .block(Block::default().title(" Verbose Data ").borders(Borders::ALL));
            f.render_widget(detail_panel, body_chunks[1]);
        }
        2 => {
            let metrics_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(8), Constraint::Min(0)])
                .split(chunks[1]);

            // 1. Session Summary Header
            let elapsed = app.combat_metrics.session_start.elapsed().as_secs_f64() / 60.0;
            let mins = elapsed.max(1.0);

            let kills = app.combat_metrics.total_kills;
            let gold = app.combat_metrics.total_gold;
            let exp = app.combat_metrics.total_exp;

            let header_text = format!(
                "Session Time: {:.1} Minutes\n\nTotal Kills: {} ({:.1} KPM)\nTotal Gold: {} ({:.0} GPM)\nTotal Exp: {} ({:.0} XPM)",
                elapsed,
                kills, kills as f64 / mins,
                gold, gold as f64 / mins,
                exp, exp as f64 / mins
            );

            let header = Paragraph::new(header_text)
                .block(Block::default().title(" Global Session Stats ").borders(Borders::ALL));
            f.render_widget(header, metrics_chunks[0]);

            let mut drops: Vec<&DropMetric> = app.combat_metrics.drops.values().collect();
            drops.sort_by(|a, b| b.drop_count.cmp(&a.drop_count));

            let header_row = Row::new(vec!["Item Name", "Total Qty", "Drop Rate", "1 in X Kills", "Since Last", "Next Est."])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .bottom_margin(1);

            let current_kills = app.combat_metrics.total_kills;

            let rows: Vec<Row> = drops.iter().map(|d| {
                let empirical_rate = d.drop_count as f64 / current_kills.max(1) as f64;
                let one_in_x = (1.0 / empirical_rate).round() as i32;
                let kills_since_last = current_kills.saturating_sub(d.kills_at_last_drop);
                
                let next_est = one_in_x - kills_since_last as i32;
                let next_str = if next_est <= 0 {
                    format!("Overdue ({})", next_est.abs())
                } else {
                    format!("in {}", next_est)
                };

                let rate_str = format!("{:.2}%", empirical_rate * 100.0);
                
                Row::new(vec![
                    d.name.clone(),
                    d.total_quantity.to_string(),
                    rate_str,
                    one_in_x.to_string(),
                    kills_since_last.to_string(),
                    next_str,
                ]).style(Style::default().fg(if next_est <= 0 { Color::Green } else { Color::White }))
            }).collect();

            let max_scroll = rows.len().saturating_sub(metrics_chunks[1].height.saturating_sub(3) as usize) as u16;
            app.scroll_metrics_y = app.scroll_metrics_y.min(max_scroll);

            let table = Table::new(rows, [
                Constraint::Percentage(30), // Name
                Constraint::Percentage(10), // Qty
                Constraint::Percentage(15), // Rate
                Constraint::Percentage(15), // 1 in X
                Constraint::Percentage(15), // Since Last
                Constraint::Percentage(15), // Next Est
            ])
            .header(header_row)
            .block(Block::default().title(" Drop Logistics (Up/Down) ").borders(Borders::ALL));

            let mut table_state = ratatui::widgets::TableState::default();
            table_state.select(Some(app.scroll_metrics_y as usize));
            f.render_stateful_widget(table, metrics_chunks[1], &mut table_state);
        }
        _ => {}
    }

    let user_items: Vec<ListItem> = app.users_in_room.iter().map(|user| {
        ListItem::new(Line::from(Span::raw(user.clone())))
    }).collect();

    let title = format!(" Users ({}) ", app.users_in_room.len());
    let users_list = List::new(user_items).block(
        Block::default().title(title).borders(Borders::ALL)
    );
    f.render_widget(users_list, chunks[2]);
}

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
    rx: Receiver<String>,
    parser: &mut ScribeParser,
) -> io::Result<()> {
    
    while !app.should_quit {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                
                    KeyCode::Enter => {
                        app.active_tab = (app.active_tab + 1) % app.tabs.len();
                        app.snap = true;
                    }

                    // Map keys dynamically based on active tab
                    KeyCode::Up => {
                        match app.active_tab {
                            0 => { app.scroll_output_y = app.scroll_output_y.saturating_sub(1); app.snap = false; },
                            1 => { app.selected_skill_index = app.selected_skill_index.saturating_sub(1); },
                            2 => { app.scroll_metrics_y = app.scroll_metrics_y.saturating_sub(1); },
                            _ => {}
                        }
                    }
                    KeyCode::Down => {
                        match app.active_tab {
                            0 => { app.scroll_output_y = app.scroll_output_y.saturating_add(1); app.snap = false; },
                            1 => { app.selected_skill_index = app.selected_skill_index.saturating_add(1); },
                            2 => { app.scroll_metrics_y = app.scroll_metrics_y.saturating_add(1); },
                            _ => {}
                        }
                    }
                    KeyCode::Left => {
                        if app.active_tab == 0 { app.scroll_output_x = app.scroll_output_x.saturating_sub(2); }
                        if app.active_tab == 1 { app.scroll_class_x = app.scroll_class_x.saturating_sub(2); }
                    }
                    KeyCode::Right => {
                        if app.active_tab == 0 { app.scroll_output_x = app.scroll_output_x.saturating_add(2); }
                        if app.active_tab == 1 { app.scroll_class_x = app.scroll_class_x.saturating_add(2); }
                    }
                    _ => {}
                }
            }
        }

        while let Ok(raw_json) = rx.try_recv() {
            let mut events = parser.parse_packet(&raw_json);
            for event in &events {
                app.handle_event(event.clone());
            }
            events.clear();
        }
    }
    Ok(())
}