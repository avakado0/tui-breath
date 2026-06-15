use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

use crate::app::{App, AppState};

pub fn draw(f: &mut Frame, app: &App) {
    let AppState::History(history_state) = &app.state else {
        return;
    };

    let area = f.size();
    let title_block = Block::default()
        .title("Session History")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    f.render_widget(title_block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 2,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(4),
    };

    if history_state.sessions.is_empty() {
        let empty_msg = Paragraph::new("No sessions recorded yet.")
            .alignment(Alignment::Center)
            .style(Style::default().dim());
        f.render_widget(empty_msg, inner);
    } else {
        let header = Row::new(vec![
            Cell::from("Date & Time").style(Style::default().bold()),
            Cell::from("Pattern").style(Style::default().bold()),
            Cell::from("Duration").style(Style::default().bold()),
            Cell::from("Hold").style(Style::default().bold()),
            Cell::from("Completion").style(Style::default().bold()),
        ]);

        let viewport_height = inner.height as usize;
        let start_idx = history_state.scroll_offset;
        let end_idx = (start_idx + viewport_height).min(history_state.sessions.len());

        let rows: Vec<Row> = history_state.sessions[start_idx..end_idx]
            .iter()
            .enumerate()
            .map(|(display_idx, entry)| {
                let actual_idx = start_idx + display_idx;
                let style = if actual_idx == history_state.selected_idx {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let date_str = entry.start_time.format("%Y-%m-%d %H:%M").to_string();
                let duration_secs = entry.duration_target;
                let mins = duration_secs / 60;
                let secs = duration_secs % 60;
                let hold_text = entry
                    .best_breath_hold_seconds
                    .map(|secs| {
                        let count = entry.breath_hold_attempt_count;
                        format!("best {secs:.1}s / {count}")
                    })
                    .unwrap_or_else(|| "--".to_string());

                Row::new(vec![
                    Cell::from(date_str),
                    Cell::from(entry.pattern_id.clone()),
                    Cell::from(format!("{}:{:02}", mins, secs)),
                    Cell::from(hold_text),
                    Cell::from(format!("{:.0}%", entry.completion_pct)),
                ])
                .style(style)
            })
            .collect();

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(22),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(20),
        ];
        let table = Table::new(rows, &widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(table, inner);
    }

    let footer = Paragraph::new("[k/↑↓/j] Navigate  [Esc] Back")
        .alignment(Alignment::Center)
        .style(Style::default().dim());

    let footer_area = Rect {
        x: area.x + 1,
        y: area.bottom().saturating_sub(2),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    f.render_widget(footer, footer_area);
}
