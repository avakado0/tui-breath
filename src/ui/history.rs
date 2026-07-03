use ratatui::prelude::*;
use ratatui::symbols;
use ratatui::widgets::{Axis, Block, Borders, Cell, Chart, Dataset, Paragraph, Row, Table};

use crate::app::{App, AppState, HistoryView, TimeFrame};
use crate::ui::trend;

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

    match history_state.view {
        HistoryView::Table => draw_table(f, history_state, inner),
        HistoryView::Trend => draw_trend(f, history_state, inner),
    }

    draw_footer(f, area, history_state);
}

fn draw_table(f: &mut Frame, history_state: &crate::app::HistoryState, inner: Rect) {
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
}

fn draw_trend(f: &mut Frame, history_state: &crate::app::HistoryState, inner: Rect) {
    let hold_series = trend::hold_series(&history_state.sessions, history_state.time_frame);
    let spd_series = trend::sessions_per_day(&history_state.sessions, history_state.time_frame);

    if hold_series.len() < 2 && spd_series.len() < 2 {
        let empty_msg = Paragraph::new("Not enough sessions yet — keep practicing.")
            .alignment(Alignment::Center)
            .style(Style::default().dim());
        f.render_widget(empty_msg, inner);
    } else {
        let chunks = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        if !hold_series.is_empty() {
            let (min_hold, max_hold) = hold_series
                .iter()
                .fold((f64::INFINITY, 0.0_f64), |(min, max), (_, val)| {
                    (min.min(*val), max.max(*val))
                });
            let hold_padding = (max_hold - min_hold).max(1.0) * 0.1;

            let dataset = Dataset::default()
                .name("Breath Hold (s)")
                .data(&hold_series)
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow));

            let chart = Chart::new(vec![dataset])
                .block(Block::default().title("Breath Hold").borders(Borders::ALL))
                .x_axis(
                    Axis::default()
                        .bounds([0.0, (hold_series.len() - 1).max(1) as f64])
                        .labels(vec![]),
                )
                .y_axis(
                    Axis::default()
                        .bounds([
                            (min_hold - hold_padding).max(0.0),
                            max_hold + hold_padding,
                        ])
                        .labels(vec!["0".into(), format!("{:.0}", max_hold).into()]),
                );

            f.render_widget(chart, chunks[0]);
        }

        if !spd_series.is_empty() {
            let max_spd = spd_series
                .iter()
                .map(|(_, val)| *val)
                .fold(0.0, f64::max);
            let spd_padding = max_spd.max(1.0) * 0.1;

            let dataset = Dataset::default()
                .name("Sessions/Day")
                .data(&spd_series)
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan));

            let chart = Chart::new(vec![dataset])
                .block(Block::default().title("Sessions Per Day").borders(Borders::ALL))
                .x_axis(
                    Axis::default()
                        .bounds([0.0, (spd_series.len() - 1).max(1) as f64])
                        .labels(vec![]),
                )
                .y_axis(
                    Axis::default()
                        .bounds([0.0, max_spd + spd_padding])
                        .labels(vec!["0".into(), format!("{:.0}", max_spd).into()]),
                );

            f.render_widget(chart, chunks[1]);
        }
    }
}

fn draw_footer(f: &mut Frame, area: Rect, history_state: &crate::app::HistoryState) {
    let frame_str = match history_state.time_frame {
        TimeFrame::SevenDays => "7d",
        TimeFrame::ThirtyDays => "30d",
        TimeFrame::All => "all",
    };

    let footer_text = match history_state.view {
        HistoryView::Table => "[k/↑↓/j] Navigate  [g] Trend view  [Esc] Back".to_string(),
        HistoryView::Trend => {
            format!("[g] Table view  [t] Time frame: {}  [Esc] Back", frame_str)
        }
    };

    let footer = Paragraph::new(footer_text)
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
