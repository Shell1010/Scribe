use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::VecDeque;

use crate::app::DpsSample;

const BRAILLE_LINE: [char; 9] = ['⠀', '⢀', '⢄', '⢆', '⢇', '⢧', '⢯', '⢟', '⣿'];

pub fn render_sparkline(
    f: &mut Frame,
    area: Rect,
    data: &VecDeque<DpsSample>,
    extract: impl Fn(&DpsSample) -> f64,
    title: &str,
    color: Color,
    y_max: f64,
) {
    if area.width < 6 || area.height < 3 || y_max <= 0.0 {
        let empty = Paragraph::new(if y_max <= 0.0 { "No data" } else { "" })
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, area);
        return;
    }

    let chart_w = area.width as usize - 2;
    let chart_h = area.height as usize - 2;

    let points: Vec<f64> = if data.len() <= chart_w {
        data.iter().map(&extract).collect()
    } else {
        let step = data.len() as f64 / chart_w as f64;
        (0..chart_w)
            .map(|i| {
                let idx = (i as f64 * step) as usize;
                extract(data.get(idx.min(data.len() - 1)).unwrap())
            })
            .collect()
    };

    let braille_rows = chart_h / 3;
    let total_rows = braille_rows * 3;

    let mut canvas = vec![vec![false; chart_w]; total_rows];
    let mut prev_row: Option<usize> = None;
    for (x, &val) in points.iter().enumerate() {
        if x >= chart_w {
            break;
        }
        let scaled = (val.min(y_max) / y_max * (total_rows as f64 - 1.0)).round() as usize;
        let row = total_rows - 1 - scaled;
        canvas[row][x] = true;
        if let Some(pr) = prev_row
            && pr != row {
                let (lo, hi) = if pr < row { (pr, row) } else { (row, pr) };
                for row in canvas.iter_mut().take(hi + 1).skip(lo) {
                    row[x] = true;
                }
            }
        prev_row = Some(row);
    }

    let max_str = format_dps(y_max);
    let label_w = max_str.len();
    let mid_str = format_dps(y_max / 2.0);

    let time_span_secs = if data.len() >= 2 {
        data.back()
            .unwrap()
            .at
            .duration_since(data.front().unwrap().at)
            .as_secs_f64()
    } else {
        0.0
    };
    let time_label_left = if time_span_secs >= 60.0 {
        format!("-{:.0}m", time_span_secs / 60.0)
    } else if time_span_secs > 0.0 {
        format!("-{:.0}s", time_span_secs)
    } else {
        String::new()
    };
    let time_axis_line = if !time_label_left.is_empty() && chart_w > time_label_left.len() + 4 {
        format!(
            "{:>w$} {:>cw$} now",
            time_label_left,
            "",
            w = label_w,
            cw = chart_w.saturating_sub(time_label_left.len() + 5),
        )
    } else {
        format!("{:>w$} {:>cw$}now", "", "", w = label_w, cw = chart_w.saturating_sub(4))
    };

    let mut lines: Vec<Line> = Vec::new();
    for braille_row in 0..braille_rows {
        let mut spans = Vec::new();
        let label = match braille_row {
            0 => max_str.clone(),
            r if r == braille_rows / 2 => mid_str.clone(),
            r if r == braille_rows - 1 => "0".to_string(),
            _ => " ".repeat(label_w),
        };
        spans.push(Span::styled(
            format!("{:>width$} ", label, width = label_w),
            Style::default().fg(Color::DarkGray),
        ));

        let base_row = braille_row * 3;
        #[allow(clippy::needless_range_loop)]
        for col in 0..chart_w {
            let mut idx: u8 = 0;
            for dy in 0..3 {
                let r = base_row + dy;
                if r < total_rows && canvas[r][col] {
                    idx |= 1 << (2 - dy);
                }
            }
            let ch = BRAILLE_LINE[idx as usize];
            let style = if ch != '⠀' {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(ch.to_string(), style));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(Span::styled(
        time_axis_line,
        Style::default().fg(Color::DarkGray),
    )));

    let graph = Paragraph::new(lines)
        .block(Block::default().title(title).borders(Borders::ALL))
        .style(Style::default().fg(color));
    f.render_widget(graph, area);
}

pub fn format_dps(val: f64) -> String {
    if val >= 1_000_000.0 {
        format!("{:.1}M", val / 1_000_000.0)
    } else if val >= 1_000.0 {
        format!("{:.1}K", val / 1_000.0)
    } else {
        format!("{:.0}", val)
    }
}
