use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::{App, AppState};
use crate::audio::AudioMode;
use crate::engine::PATTERNS;

pub fn draw(f: &mut Frame, app: &App) {
    let AppState::Menu(menu_state) = &app.state else {
        return;
    };

    let area = f.size();
    let title_block = Block::default()
        .title("TUI BREATH - Breathing Guide")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    f.render_widget(title_block, area);

    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Title
    let title = Paragraph::new("Select a Breathing Pattern")
        .alignment(Alignment::Center)
        .style(Style::default().bold());
    let title_area = Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: 2,
    };
    f.render_widget(title, title_area);

    // Pattern list
    let patterns: Vec<ListItem> = PATTERNS
        .iter()
        .enumerate()
        .map(|(idx, pattern)| {
            let content = format!("{}\n  {}", pattern.display_name, pattern.description);
            let item = ListItem::new(content);
            if idx == menu_state.selected_pattern_idx {
                item.style(Style::default().fg(Color::Cyan).bold())
            } else {
                item
            }
        })
        .collect();

    let list = List::new(patterns)
        .block(Block::default().borders(Borders::ALL).title("Patterns"))
        .highlight_symbol("> ");

    let list_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 3,
        width: inner_area.width,
        height: inner_area.height.saturating_sub(6),
    };
    f.render_widget(list, list_area);

    // Update banner and audio device notice
    if let Some(ref version) = app.update_available {
        let banner = Paragraph::new(format!(
            " Update available: v{}  [U] install & exit  [any key] skip ",
            version
        ))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).bold());
        let banner_area = Rect {
            x: inner_area.x,
            y: area.bottom().saturating_sub(4),
            width: inner_area.width,
            height: 1,
        };
        f.render_widget(banner, banner_area);
    }

    #[cfg(feature = "audio")]
    if !app.audio_device_available {
        let update_offset = if app.update_available.is_some() { 1 } else { 0 };
        let notice = Paragraph::new("Tone mode unavailable — no audio device detected")
            .alignment(Alignment::Center)
            .style(Style::default().dim());
        let notice_area = Rect {
            x: inner_area.x,
            y: area.bottom().saturating_sub(4 - update_offset),
            width: inner_area.width,
            height: 1,
        };
        f.render_widget(notice, notice_area);
    }

    // Workout toggle
    let workout_label = if menu_state.workout_mode {
        "Workout: ON "
    } else {
        "Workout: OFF"
    };
    let workout_style = if menu_state.workout_mode {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().dim()
    };
    let workout_area = Rect {
        x: inner_area.x,
        y: area.bottom().saturating_sub(3),
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(
        Paragraph::new(format!("[w] {}", workout_label))
            .alignment(Alignment::Center)
            .style(workout_style),
        workout_area,
    );

    // Footer
    let audio_label = match app.audio_mode {
        AudioMode::Off => "audio off",
        AudioMode::Beep => "audio beep",
        #[cfg(feature = "audio")]
        AudioMode::Tone => "audio tone",
    };
    let footer = Paragraph::new(format!(
        "[k/↑] Up  [j/↓] Down  [Enter] Select  [h] History  [b] {}  [q] Quit",
        audio_label
    ))
    .alignment(Alignment::Center)
    .style(Style::default().dim());
    let footer_area = Rect {
        x: inner_area.x,
        y: area.bottom().saturating_sub(2),
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(footer, footer_area);
}
