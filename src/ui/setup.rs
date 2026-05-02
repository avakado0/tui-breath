use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

use crate::app::{App, AppState, SetupField, SetupState};

pub fn draw(f: &mut Frame, app: &App) {
    let AppState::Setup(setup_state) = &app.state else {
        return;
    };

    let area = f.size();
    let title_block = Block::default()
        .title("Session Setup")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    f.render_widget(title_block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(4),
    };

    // Duration field
    let duration_label = "Session Duration (minutes):";
    let duration_value = format!("{}", setup_state.duration_minutes);
    let duration_style = if setup_state.selected_field == SetupField::Duration {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default()
    };

    let duration_text = format!("{} {}", duration_label, duration_value);
    let duration_para = Paragraph::new(duration_text).style(duration_style);
    let duration_rect = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 2,
    };
    f.render_widget(duration_para, duration_rect);

    // Tempo field
    let tempo_label = "Tempo Multiplier:";
    let tempo_value = format!("{:.1}x", setup_state.tempo);
    let tempo_style = if setup_state.selected_field == SetupField::Tempo {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default()
    };

    let tempo_text = format!("{} {}", tempo_label, tempo_value);
    let tempo_para = Paragraph::new(tempo_text).style(tempo_style);
    let tempo_rect = Rect {
        x: inner.x,
        y: inner.y + 3,
        width: inner.width,
        height: 2,
    };
    f.render_widget(tempo_para, tempo_rect);

    // Instructions
    let instructions = Paragraph::new(
        "[Tab] Switch  [+/-] Adjust  [Enter] Start  [Esc] Back",
    )
    .alignment(Alignment::Center)
    .style(Style::default().dim());

    let instructions_rect = Rect {
        x: inner.x,
        y: area.bottom().saturating_sub(2),
        width: inner.width,
        height: 1,
    };
    f.render_widget(instructions, instructions_rect);
}
