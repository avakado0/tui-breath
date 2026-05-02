use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, AppState, SetupField};
use crate::engine::{patterns::PhaseStyle, PATTERNS};

pub fn draw(f: &mut Frame, app: &App) {
    let AppState::Setup(setup_state) = &app.state else {
        return;
    };

    let area = f.size();
    let pattern = &PATTERNS[setup_state.pattern_idx];

    let outer_block = Block::default()
        .title("Session Setup")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(90, 70, 130)));
    f.render_widget(outer_block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(4),
    };

    // Pattern name
    f.render_widget(
        Paragraph::new(pattern.display_name)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(200, 175, 255)).bold()),
        Rect { x: inner.x, y: inner.y, width: inner.width, height: 1 },
    );

    // Phase summary: "Inhale 4s  ·  Hold 7s  ·  Exhale 8s"
    let phase_summary = pattern.phases.iter()
        .map(|p| format!("{} {}s", p.name, p.duration_secs as u32))
        .collect::<Vec<_>>()
        .join("  ·  ");
    f.render_widget(
        Paragraph::new(phase_summary)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(130, 120, 155))),
        Rect { x: inner.x, y: inner.y + 1, width: inner.width, height: 1 },
    );

    // ── Two cards ──────────────────────────────────────
    let card_y = inner.y + 3;
    let card_h: u16 = 7;
    let card_gap: u16 = 2;
    let left_w = inner.width / 2 - card_gap / 2;
    let right_x = inner.x + inner.width / 2 + card_gap / 2;
    let right_w = inner.width.saturating_sub(inner.width / 2 + card_gap / 2);

    let dur_selected = setup_state.selected_field == SetupField::Duration;
    let speed_selected = setup_state.selected_field == SetupField::Tempo;

    // Duration card
    let dur_border = if dur_selected { Color::Cyan } else { Color::Rgb(70, 60, 100) };
    let dur_rect = Rect { x: inner.x, y: card_y, width: left_w, height: card_h };
    f.render_widget(
        Block::default()
            .title(" Duration ")
            .title_style(Style::default().fg(if dur_selected { Color::Cyan } else { Color::Rgb(130, 120, 155) }))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(dur_border)),
        dur_rect,
    );

    let dur_inner = Rect {
        x: dur_rect.x + 1,
        y: dur_rect.y + 2,
        width: left_w.saturating_sub(2),
        height: card_h.saturating_sub(4),
    };

    let base_cycle_secs: f64 = pattern.phases.iter().map(|p| p.duration_secs).sum();
    let session_secs = setup_state.duration_units as f64 * base_cycle_secs / setup_state.tempo;
    let session_mins = session_secs / 60.0;

    f.render_widget(
        Paragraph::new(format!("{} units  (≈ {:.1} min)", setup_state.duration_units, session_mins))
            .alignment(Alignment::Center)
            .style(if dur_selected {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            }),
        Rect { x: dur_inner.x, y: dur_inner.y, width: dur_inner.width, height: 1 },
    );

    f.render_widget(
        Paragraph::new("1 unit = 1 complete breathing cycle")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(110, 100, 135)).dim()),
        Rect { x: dur_inner.x, y: dur_inner.y + 1, width: dur_inner.width, height: 1 },
    );
    f.render_widget(
        Paragraph::new(format!("≈ {} breathing cycles", setup_state.duration_units))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(90, 80, 115)).dim()),
        Rect { x: dur_inner.x, y: dur_inner.y + 2, width: dur_inner.width, height: 1 },
    );

    // Speed card
    let speed_border = if speed_selected { Color::Rgb(255, 185, 80) } else { Color::Rgb(70, 60, 100) };
    let speed_rect = Rect { x: right_x, y: card_y, width: right_w, height: card_h };
    f.render_widget(
        Block::default()
            .title(" Breathing Speed ")
            .title_style(Style::default().fg(if speed_selected { Color::Rgb(255, 185, 80) } else { Color::Rgb(130, 120, 155) }))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(speed_border)),
        speed_rect,
    );

    let speed_inner = Rect {
        x: speed_rect.x + 1,
        y: speed_rect.y + 2,
        width: right_w.saturating_sub(2),
        height: card_h.saturating_sub(4),
    };

    f.render_widget(
        Paragraph::new(format!("{:.1}×", setup_state.tempo))
            .alignment(Alignment::Center)
            .style(if speed_selected {
                Style::default().fg(Color::Rgb(255, 185, 80)).bold()
            } else {
                Style::default().fg(Color::White)
            }),
        Rect { x: speed_inner.x, y: speed_inner.y, width: speed_inner.width, height: 1 },
    );

    f.render_widget(
        Paragraph::new(tempo_description(setup_state.tempo))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(110, 100, 135)).dim()),
        Rect { x: speed_inner.x, y: speed_inner.y + 1, width: speed_inner.width, height: 1 },
    );

    // Example: show first phase adjusted duration
    let first = &pattern.phases[0];
    let adj = first.duration_secs / setup_state.tempo;
    f.render_widget(
        Paragraph::new(format!("{} = {:.1}s per phase", first.name, adj))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(90, 80, 115)).dim()),
        Rect { x: speed_inner.x, y: speed_inner.y + 2, width: speed_inner.width, height: 1 },
    );

    // ── Phase duration bar ──────────────────────────────
    let bar_y = card_y + card_h + 1;

    f.render_widget(
        Paragraph::new(format!("Phase durations at {:.1}×", setup_state.tempo))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(130, 120, 155))),
        Rect { x: inner.x, y: bar_y, width: inner.width, height: 1 },
    );

    let total_adj: f64 = base_cycle_secs / setup_state.tempo;
    let bar_w = (inner.width as f64 * 0.88) as usize;
    let bar_x = inner.x + ((inner.width as f64 * 0.06) as u16);

    // Proportional colored block segments
    let mut bar_spans: Vec<Span> = Vec::new();
    for (i, phase) in pattern.phases.iter().enumerate() {
        let adj_dur = phase.duration_secs / setup_state.tempo;
        let seg_w = ((adj_dur / total_adj) * bar_w as f64).round() as usize;
        if seg_w == 0 { continue; }
        let color = phase_color(&phase.style);
        // Dim separator between segments
        if i > 0 {
            bar_spans.push(Span::styled("│", Style::default().fg(Color::Rgb(40, 35, 55))));
        }
        let seg_w_adj = if i > 0 { seg_w.saturating_sub(1) } else { seg_w };
        bar_spans.push(Span::styled("█".repeat(seg_w_adj), Style::default().fg(color)));
    }
    f.render_widget(
        Paragraph::new(vec![Line::from(bar_spans)]),
        Rect { x: bar_x, y: bar_y + 1, width: bar_w as u16, height: 1 },
    );

    // Legend: ■ Inhale 4.0s  ■ Hold 7.0s  ■ Exhale 8.0s
    let mut legend_spans: Vec<Span> = Vec::new();
    for (i, phase) in pattern.phases.iter().enumerate() {
        let adj_dur = phase.duration_secs / setup_state.tempo;
        let color = phase_color(&phase.style);
        if i > 0 {
            legend_spans.push(Span::raw("   "));
        }
        legend_spans.push(Span::styled("■ ", Style::default().fg(color)));
        legend_spans.push(Span::styled(
            format!("{} {:.1}s", phase.name, adj_dur),
            Style::default().fg(Color::Rgb(150, 140, 170)),
        ));
    }
    f.render_widget(
        Paragraph::new(vec![Line::from(legend_spans)]).alignment(Alignment::Center),
        Rect { x: inner.x, y: bar_y + 2, width: inner.width, height: 1 },
    );

    // ── Pattern info + waveform in empty space ──────────
    let info_y = bar_y + 4;
    let footer_y = area.bottom().saturating_sub(2);
    let available = footer_y.saturating_sub(info_y);

    if available >= 3 {
        let (desc, best_for) = pattern_info(pattern.display_name);
        f.render_widget(
            Paragraph::new(desc)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Rgb(160, 150, 185))),
            Rect { x: inner.x, y: info_y, width: inner.width, height: 1 },
        );
        if available >= 4 {
            f.render_widget(
                Paragraph::new(format!("Best for: {}", best_for))
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Rgb(110, 100, 135)).dim()),
                Rect { x: inner.x, y: info_y + 1, width: inner.width, height: 1 },
            );
        }

        // Waveform: filled color wave showing 2 breathing cycles
        let wave_top = info_y + 3;
        let wave_h = footer_y.saturating_sub(wave_top + 1);
        if wave_h >= 3 {
            let lines = build_waveform(inner.width, pattern.phases, setup_state.tempo, wave_h);
            f.render_widget(
                Paragraph::new(lines),
                Rect { x: inner.x, y: wave_top, width: inner.width, height: wave_h },
            );
        }
    }

    // Footer
    f.render_widget(
        Paragraph::new("[Tab] Switch field   [↑/↓] or [+/-] Adjust   [Enter] Start   [Esc] Back")
            .alignment(Alignment::Center)
            .style(Style::default().dim()),
        Rect {
            x: inner.x,
            y: footer_y,
            width: inner.width,
            height: 1,
        },
    );
}

fn build_waveform(width: u16, phases: &[crate::engine::patterns::Phase], tempo: f64, wave_h: u16) -> Vec<Line<'static>> {
    let cols = width as usize;
    let h = wave_h as usize;
    let total_adj: f64 = phases.iter().map(|p| p.duration_secs / tempo).sum();
    let cycles = 2usize;
    let cycle_cols = (cols / cycles).max(1);

    // Build height+color per column
    let mut col_data: Vec<(usize, Color)> = vec![(0, Color::Reset); cols];

    for cycle in 0..cycles {
        for c in 0..cycle_cols {
            let col = cycle * cycle_cols + c;
            if col >= cols { break; }
            let t = (c as f64 / cycle_cols as f64) * total_adj;
            let mut elapsed = 0.0f64;
            for phase in phases.iter() {
                let dur = phase.duration_secs / tempo;
                if t < elapsed + dur || elapsed + dur >= total_adj {
                    let p = ((t - elapsed) / dur).clamp(0.0, 1.0);
                    let h_frac = match phase.style {
                        PhaseStyle::Rising  => p,
                        PhaseStyle::Falling => 1.0 - p,
                        PhaseStyle::Steady  => 1.0,
                    };
                    let color = phase_color(&phase.style);
                    col_data[col] = ((h_frac * h as f64) as usize, color);
                    break;
                }
                elapsed += dur;
            }
        }
    }

    // Render rows top→bottom; fill from bottom up
    let mut lines: Vec<Line<'static>> = Vec::with_capacity(h);
    for row in 0..h {
        let rows_from_bottom = h - 1 - row;
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut run = String::new();
        let mut run_color = Color::Reset;

        for col in 0..cols {
            let (cell_h, color) = col_data[col];
            let ch = if rows_from_bottom < cell_h { '█' } else { ' ' };
            let cur_color = if ch == '█' { color } else { Color::Reset };

            if cur_color == run_color {
                run.push(ch);
            } else {
                if !run.is_empty() {
                    spans.push(Span::styled(run.clone(), Style::default().fg(run_color)));
                    run.clear();
                }
                run.push(ch);
                run_color = cur_color;
            }
        }
        if !run.is_empty() {
            spans.push(Span::styled(run, Style::default().fg(run_color)));
        }
        lines.push(Line::from(spans));
    }
    lines
}

fn pattern_info(name: &str) -> (&'static str, &'static str) {
    match name {
        "4-7-8 Breathing" => (
            "Extended exhale activates the parasympathetic nervous system.",
            "anxiety, falling asleep, acute stress",
        ),
        "Box Breathing" => (
            "Equal phases build rhythmic control. Used by Navy SEALs and athletes.",
            "focus, performance pressure, emotional regulation",
        ),
        "Diaphragmatic Breathing" => (
            "Engages the diaphragm fully, maximizing oxygen exchange with minimal effort.",
            "daily practice, energy, reducing shallow breathing",
        ),
        _ => ("Breathe slowly and deliberately.", "relaxation"),
    }
}

fn phase_color(style: &PhaseStyle) -> Color {
    match style {
        PhaseStyle::Rising  => Color::Rgb(0, 200, 220),
        PhaseStyle::Steady  => Color::Rgb(220, 190, 0),
        PhaseStyle::Falling => Color::Rgb(0, 190, 100),
    }
}

fn tempo_description(tempo: f64) -> &'static str {
    if tempo >= 1.85      { "very fast — phases nearly halved" }
    else if tempo >= 1.4  { "fast" }
    else if tempo >= 1.15 { "slightly fast" }
    else if tempo >= 0.85 { "normal pace" }
    else if tempo >= 0.6  { "slightly slow" }
    else if tempo >= 0.4  { "slow" }
    else                  { "very slow — phases nearly doubled" }
}
