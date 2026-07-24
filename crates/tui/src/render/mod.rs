use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub mod clipboard;
pub mod input;
pub mod sparkline;
pub mod tabs;

use tabs::*;

pub fn highlight_search<'a>(text: &'a str, query: &str, is_match_line: bool) -> Vec<Span<'a>> {
    if query.is_empty() || !is_match_line {
        return vec![Span::raw(text)];
    }
    let lower = text.to_lowercase();
    let q_lower = query.to_lowercase();
    let mut spans = Vec::new();
    let mut last = 0;
    for (start, _) in lower.match_indices(&q_lower) {
        if start > last {
            spans.push(Span::raw(&text[last..start]));
        }
        spans.push(Span::styled(
            &text[start..start + query.len()],
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ));
        last = start + query.len();
    }
    if last < text.len() {
        spans.push(Span::raw(&text[last..]));
    }
    spans
}

pub fn ui(f: &mut Frame, app: &mut App) {
    app.clear_stale_status();

    let area = f.area();

    if app.split_left == 0 {
        app.split_left = area.width / 6;
    }

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let max_left = area.width.saturating_sub(14);
    app.split_left = app.split_left.min(max_left).max(12);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(app.split_left),
            Constraint::Min(10),
        ])
        .split(outer[0]);

    let tab_items: Vec<ListItem> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, &tab_name)| {
            let style = if i == app.active_tab {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(
                format!(" {} {} ", i + 1, tab_name),
                style,
            )))
        })
        .collect();

    let tabs_list = List::new(tab_items).block(
        Block::default().title(" Menu (Enter) ").borders(Borders::ALL),
    );
    f.render_widget(tabs_list, chunks[0]);

    match app.active_tab {
        0 => render_output_log(f, app, chunks[1]),
        1 => render_class_data(f, app, chunks[1]),
        2 => render_combat_metrics(f, app, chunks[1]),
        3 => render_ultra_mechanics(f, app, chunks[1]),
        4 => render_party_status(f, app, chunks[1]),
        _ => {}
    }

    let status_text = if app.show_help {
        "? : close help"
    } else if app.search_mode {
        "/ — Type query, Enter: confirm, Esc: cancel"
    } else if app.filter_mode {
        "/ — Type filter, Enter: confirm, Esc: cancel"
    } else if let Some((msg, _)) = &app.status_message {
        msg.as_str()
    } else {
        match app.active_tab {
            0 => "q: quit | Enter/1-5: tabs | ↑/↓: nav | ←/→: scroll | PgUp/PgDn: page | c: copy | /: search | Space: pause | g/G: top/bottom",
            1 => "q: quit | Enter/1-5: tabs | ↑/↓: skill | ←/→: scroll | c: copy | /: filter skills",
            2 => "q: quit | Enter/1-5: tabs | ↑/↓: drops | d: toggle graph | c: copy | /: filter drops",
            3 => "q: quit | Enter/1-5: tabs | ↑/↓: role | ←/→: strategy | ?: help",
            4 => "q: quit | Enter/1-5: tabs | ↑/↓: entity | c: copy | /: filter",
            _ => "q: quit | Enter/1-5: tabs | ?: help",
        }
    };
    let status = Paragraph::new(status_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, outer[1]);

    if app.show_help {
        render_help_modal(f, area);
    }
}

fn render_help_modal(f: &mut Frame, area: Rect) {
    let popup_w = 62.min(area.width.saturating_sub(4));
    let popup_h = 28.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(popup_w)) / 2;
    let y = (area.height.saturating_sub(popup_h)) / 2;
    let popup = Rect::new(x, y, popup_w, popup_h);

    let block = Block::default()
        .title(" Help (? to close) ")
        .borders(Borders::ALL)
        .style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Black),
        );
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let sections = [
        ("General", vec![
            "q .............. Quit",
            "Enter ........... Next tab",
            "1-5 ............. Jump to tab",
            "Shift-Tab ....... Previous tab",
            "? ............... Toggle this help",
        ]),
        ("Output Log", vec![
            "/ ............... Search log",
            "n / N ........... Next / Prev match",
            "↑ / ↓ ........... Scroll one line",
            "PgUp / PgDn .... Scroll one page",
            "g / G ........... Top / Bottom",
            "← / → ........... Horizontal scroll",
            "Space ........... Pause / Resume",
            "c ............... Copy line",
        ]),
        ("Class Data", vec![
            "↑ / ↓ ........... Navigate skills",
            "← / → ........... Horizontal scroll",
            "c ............... Copy skill JSON",
            "/ ............... Filter skills",
        ]),
        ("Combat Metrics", vec![
            "↑ / ↓ ........... Scroll drops",
            "d ............... Toggle DPS / Drops",
            "c ............... Copy drop info",
            "/ ............... Filter drops",
        ]),
        ("Ultra Mechanics", vec![
            "↑ / ↓ ........... Cycle role",
            "← / → ........... Cycle strategy",
        ]),
        ("Party Status", vec![
            "↑ / ↓ ........... Navigate entities",
            "c ............... Copy entity info",
            "/ ............... Filter by name",
        ]),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (title, keys) in sections {
        lines.push(Line::from(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        for key in keys {
            lines.push(Line::from(Span::styled(
                format!("   {}", key),
                Style::default().fg(Color::White),
            )));
        }
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines).scroll((0, 0));
    f.render_widget(paragraph, inner);
}
