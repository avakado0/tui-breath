use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::{App, AppState};
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

    // Footer
    let beep_status = if app.beeper.is_enabled() { "🔊" } else { "🔇" };
    let footer = Paragraph::new(format!(
        "[k/↑] Up  [j/↓] Down  [Enter] Select  [h] History  [b] {} Beep  [q] Quit",
        beep_status
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
