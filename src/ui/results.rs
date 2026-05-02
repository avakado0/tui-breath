use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Table, Row, Cell};

use crate::app::{App, AppState};

pub fn draw(f: &mut Frame, app: &App) {
    let AppState::Results(results_state) = &app.state else {
        return;
    };

    let engine = &results_state.manager.engine;
    let pattern = engine.pattern;
    let area = f.size();

    let title_block = Block::default()
        .title("Session Complete")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    f.render_widget(title_block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(5),
    };

    // Stats table
    let elapsed_secs = engine.total_elapsed_secs as u32;
    let elapsed_mins = elapsed_secs / 60;
    let elapsed_secs_remainder = elapsed_secs % 60;

    let target_secs = engine.duration_target_secs as u32;
    let target_mins = target_secs / 60;
    let target_secs_remainder = target_secs % 60;

    let rows = vec![
        Row::new(vec![
            Cell::from("Pattern"),
            Cell::from(pattern.display_name).style(Style::default().fg(Color::Cyan)),
        ]),
        Row::new(vec![
            Cell::from("Duration"),
            Cell::from(format!(
                "{}:{:02} / {}:{:02}",
                elapsed_mins, elapsed_secs_remainder, target_mins, target_secs_remainder
            ))
            .style(Style::default().fg(Color::Green)),
        ]),
        Row::new(vec![
            Cell::from("Cycles Completed"),
            Cell::from(format!("{}", engine.cycle_count))
                .style(Style::default().fg(Color::Yellow)),
        ]),
        Row::new(vec![
            Cell::from("Pauses"),
            Cell::from(format!("{}", engine.pause_count))
                .style(Style::default().fg(Color::Magenta)),
        ]),
    ];

    let table = Table::new(rows, &[Constraint::Percentage(40), Constraint::Percentage(60)])
        .block(Block::default().borders(Borders::ALL).title("Session Metrics"));

    let table_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 8,
    };
    f.render_widget(table, table_area);

    // Progress bar
    let completion = engine.completion_percent() as u16;
    let gauge = Gauge::default()
        .block(Block::default().title("Completion"))
        .gauge_style(Style::default().fg(Color::Blue))
        .percent(completion);

    let gauge_area = Rect {
        x: inner.x,
        y: inner.y + 9,
        width: inner.width,
        height: 2,
    };
    f.render_widget(gauge, gauge_area);

    // Footer
    let footer = Paragraph::new("[s] Save & Continue  [Enter] Main Menu  [Esc] Quit")
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
