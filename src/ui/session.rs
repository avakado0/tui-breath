use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

use crate::app::{App, AppState, SessionMode};
use crate::engine::patterns::{Channel, PhaseStyle};

pub fn draw(f: &mut Frame, app: &App) {
    let AppState::Session(session_state) = &app.state else {
        return;
    };

    let engine = &session_state.manager.engine;
    let pattern = engine.pattern;
    let area = f.size();

    let title = match &session_state.mode {
        SessionMode::Breathing if engine.is_paused => format!("{} [PAUSED]", pattern.display_name),
        SessionMode::Holding(_) => format!("{} [BREATH HOLD]", pattern.display_name),
        SessionMode::Breathing => pattern.display_name.to_string(),
    };

    let title_block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    f.render_widget(title_block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 2,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(5),
    };

    match &session_state.mode {
        SessionMode::Breathing => render_breathing_view(f, inner, app),
        SessionMode::Holding(runtime) => render_hold_view(f, inner, app, runtime.elapsed_secs),
    }

    let beep_status = if app.beeper.is_enabled() {
        "🔊"
    } else {
        "🔇"
    };
    let workout_hint = if session_state.workout_mode {
        "  [m] Movements"
    } else {
        ""
    };
    let footer = match &session_state.mode {
        SessionMode::Breathing if engine.is_paused => {
            format!(
                "[p] Resume  [h] Hold  [e] End  [b] {} Beep{}  [q] Quit",
                beep_status, workout_hint
            )
        }
        SessionMode::Breathing => {
            format!(
                "[p] Pause  [h] Hold  [e] End  [b] {} Beep{}  [q] Quit",
                beep_status, workout_hint
            )
        }
        SessionMode::Holding(_) => {
            format!(
                "[h] End Hold  [e] End Session  [b] {} Beep{}  [q] Quit",
                beep_status, workout_hint
            )
        }
    };

    let footer_para = Paragraph::new(footer)
        .alignment(Alignment::Center)
        .style(Style::default().dim());

    let footer_area = Rect {
        x: area.x + 1,
        y: area.bottom().saturating_sub(1),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    f.render_widget(footer_para, footer_area);
}

fn render_breathing_view(f: &mut Frame, inner: Rect, app: &App) {
    let AppState::Session(session_state) = &app.state else {
        return;
    };
    let engine = &session_state.manager.engine;

    let animation_height = inner.height.saturating_sub(8).max(5);
    let animation_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: animation_height,
    };

    render_breathing_circle(f, animation_area, app);

    let current_phase = engine.current_phase();
    let remaining = (engine.phase_remaining() * 10.0).ceil() / 10.0;

    let (cr, cg, cb) = get_anim_color(app, &current_phase.style);
    let phase_color = Color::Rgb(cr, cg, cb);

    let phase_label_owned;
    let phase_label = if let Some(anim) = &app.session_animator {
        phase_label_owned = anim.phase_label.get().to_string();
        phase_label_owned.as_str()
    } else {
        current_phase.name
    };

    let channel_str = match current_phase.channel {
        Some(Channel::Nose) => "░ nose ░",
        Some(Channel::Mouth) => "░ mouth ░",
        None => "",
    };

    let phase_text = Text::from(vec![
        Line::from(format!("*** {} ***", phase_label))
            .style(Style::default().fg(phase_color).bold()),
        Line::from(format!("{remaining:.1}s remaining"))
            .style(Style::default().fg(phase_color).bold()),
        Line::from(""),
        Line::from(channel_str).style(Style::default().fg(Color::Rgb(cr / 2, cg / 2, cb / 2))),
    ]);
    let phase_para = Paragraph::new(phase_text).alignment(Alignment::Center);

    let phase_area = Rect {
        x: inner.x,
        y: inner.y + animation_height + 1,
        width: inner.width,
        height: 4,
    };
    f.render_widget(phase_para, phase_area);

    render_progress_stats(f, inner, app, animation_height + 5);
}

fn render_hold_view(f: &mut Frame, inner: Rect, app: &App, hold_elapsed: f64) {
    let AppState::Session(session_state) = &app.state else {
        return;
    };

    let hold_color = hold_color(app);
    let dim_color = dim_color(hold_color, 2);
    let bg_color = Color::Rgb(38, 7, 18);

    let art_height = inner.height.saturating_sub(10).max(7);
    let art_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: art_height,
    };

    render_hold_art(f, art_area, hold_color, bg_color);

    let best_hold = session_state
        .manager
        .best_hold_seconds()
        .map(|secs| format!("Best Hold: {secs:.1}s"))
        .unwrap_or_else(|| "Best Hold: --".to_string());

    let hold_text = Text::from(vec![
        Line::from("*** Breath Hold ***").style(Style::default().fg(hold_color).bold()),
        Line::from(format!("{hold_elapsed:.1}s")).style(Style::default().fg(hold_color).bold()),
        Line::from(format!(
            "Attempts: {}",
            session_state.manager.hold_attempt_count()
        ))
        .style(Style::default().fg(dim_color)),
        Line::from(best_hold).style(Style::default().fg(dim_color)),
    ]);
    let hold_para = Paragraph::new(hold_text).alignment(Alignment::Center);

    let hold_area = Rect {
        x: inner.x,
        y: inner.y + art_height + 1,
        width: inner.width,
        height: 4,
    };
    f.render_widget(hold_para, hold_area);

    render_progress_stats(f, inner, app, art_height + 5);
}

fn render_progress_stats(f: &mut Frame, inner: Rect, app: &App, offset_y: u16) {
    let AppState::Session(session_state) = &app.state else {
        return;
    };

    let engine = &session_state.manager.engine;
    let elapsed_secs = engine.total_elapsed_secs as u32;
    let elapsed_mins = elapsed_secs / 60;
    let elapsed_secs_remainder = elapsed_secs % 60;
    let target_secs = engine.duration_target_secs as u32;
    let target_mins = target_secs / 60;
    let target_secs_remainder = target_secs % 60;

    let stats_text = format!(
        "Cycle: {}   Elapsed: {}:{:02} / {}:{:02}",
        engine.cycle_count,
        elapsed_mins,
        elapsed_secs_remainder,
        target_mins,
        target_secs_remainder
    );

    let stats_para = Paragraph::new(stats_text)
        .alignment(Alignment::Center)
        .style(Style::default().dim());

    let stats_area = Rect {
        x: inner.x,
        y: inner.y + offset_y,
        width: inner.width,
        height: 1,
    };
    f.render_widget(stats_para, stats_area);

    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::Rgb(60, 120, 150)))
        .percent(engine.completion_percent() as u16);

    let gauge_area = Rect {
        x: inner.x,
        y: inner.y + offset_y + 1,
        width: inner.width,
        height: 2,
    };
    f.render_widget(gauge, gauge_area);
}

fn get_anim_color(app: &App, style: &PhaseStyle) -> (u8, u8, u8) {
    if let Some(anim) = &app.session_animator {
        (
            *anim.color_r as u8,
            *anim.color_g as u8,
            *anim.color_b as u8,
        )
    } else {
        match style {
            PhaseStyle::Rising => (0, 255, 255),
            PhaseStyle::Steady => (255, 230, 0),
            PhaseStyle::Falling => (0, 220, 100),
        }
    }
}

fn hold_color(app: &App) -> Color {
    if let Some(anim) = &app.session_animator {
        let pulse = *anim.hold_pulse;
        return Color::Rgb(
            ((*anim.color_r) * pulse) as u8,
            ((*anim.color_g) * pulse) as u8,
            ((*anim.color_b) * pulse) as u8,
        );
    }

    Color::Rgb(255, 120, 140)
}

fn dim_color(color: Color, divisor: u8) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(r / divisor, g / divisor, b / divisor),
        _ => Color::DarkGray,
    }
}

fn render_hold_art(f: &mut Frame, area: Rect, hold_color: Color, bg_color: Color) {
    let dim = dim_color(hold_color, 3);
    let glow = dim_color(hold_color, 6);
    let art = vec![
        Line::from(Span::styled(
            " ".repeat(area.width as usize),
            Style::default().bg(bg_color),
        )),
        Line::from(vec![Span::styled(
            "            ░▓▓▓░     ░▓▓▓░",
            Style::default().fg(glow).bg(bg_color),
        )]),
        Line::from(vec![Span::styled(
            "          ░██████░   ░██████░",
            Style::default().fg(dim).bg(bg_color),
        )]),
        Line::from(vec![Span::styled(
            "         ░████████░ ░████████░",
            Style::default().fg(hold_color).bg(bg_color),
        )]),
        Line::from(vec![Span::styled(
            "         ██████████ ██████████",
            Style::default().fg(hold_color).bg(bg_color),
        )]),
        Line::from(vec![Span::styled(
            "         █████████████████████",
            Style::default().fg(hold_color).bg(bg_color),
        )]),
        Line::from(vec![Span::styled(
            "          ████████   ████████",
            Style::default().fg(dim).bg(bg_color),
        )]),
        Line::from(vec![Span::styled(
            "           ▓█████     █████▓",
            Style::default().fg(dim).bg(bg_color),
        )]),
        Line::from(vec![Span::styled(
            "             ▓▓▓       ▓▓▓",
            Style::default().fg(glow).bg(bg_color),
        )]),
    ];

    let paragraph = Paragraph::new(art)
        .alignment(Alignment::Center)
        .style(Style::default().bg(bg_color));
    f.render_widget(paragraph, area);
}

pub fn render_breathing_circle_for(
    f: &mut Frame,
    area: Rect,
    engine: &crate::engine::breathing::BreathingEngine,
    animator: Option<&crate::animator::SessionAnimator>,
) {
    let phase = engine.current_phase();
    let progress = crate::animator::cubic_in_out(engine.phase_progress());
    let (cr, cg, cb) = if let Some(anim) = animator {
        let (r, g, b) = (*anim.color_r, *anim.color_g, *anim.color_b);
        if matches!(phase.style, PhaseStyle::Steady) {
            let pulse = *anim.hold_pulse;
            ((r * pulse) as u8, (g * pulse) as u8, (b * pulse) as u8)
        } else {
            (r as u8, g as u8, b as u8)
        }
    } else {
        match phase.style {
            PhaseStyle::Rising => (0, 255, 255),
            PhaseStyle::Steady => (255, 230, 0),
            PhaseStyle::Falling => (0, 220, 100),
        }
    };
    render_circle_with_color(f, area, engine, cr, cg, cb, progress);
}

fn render_breathing_circle(f: &mut Frame, area: Rect, app: &App) {
    let AppState::Session(session_state) = &app.state else {
        return;
    };
    let engine = &session_state.manager.engine;
    let phase = engine.current_phase();
    let progress = crate::animator::cubic_in_out(engine.phase_progress());
    let (cr, cg, cb) = if let Some(anim) = &app.session_animator {
        let (r, g, b) = (*anim.color_r, *anim.color_g, *anim.color_b);
        if matches!(phase.style, PhaseStyle::Steady) {
            let pulse = *anim.hold_pulse;
            ((r * pulse) as u8, (g * pulse) as u8, (b * pulse) as u8)
        } else {
            (r as u8, g as u8, b as u8)
        }
    } else {
        match phase.style {
            PhaseStyle::Rising => (0, 255, 255),
            PhaseStyle::Steady => (255, 230, 0),
            PhaseStyle::Falling => (0, 220, 100),
        }
    };
    render_circle_with_color(f, area, engine, cr, cg, cb, progress);
}

pub fn render_circle_with_color(
    f: &mut Frame,
    area: Rect,
    engine: &crate::engine::breathing::BreathingEngine,
    cr: u8,
    cg: u8,
    cb: u8,
    progress: f64,
) {
    let h = area.height as f64;
    let w = area.width as f64;
    let cy = h / 2.0;
    let cx = w / 2.0;
    let max_r = cy.min(cx / 2.0) * 0.95;

    let ratio = crate::engine::patterns::fill_ratio(
        engine.pattern.phases,
        engine.current_phase_idx,
        progress,
    );
    let base_r = ratio * max_r;
    let glow_r = base_r + 2.0;

    let fill_color = Color::Rgb(cr, cg, cb);
    let edge_color = Color::Rgb(
        (cr as u16 * 2 / 3) as u8,
        (cg as u16 * 2 / 3) as u8,
        (cb as u16 * 2 / 3) as u8,
    );
    let glow_color = Color::Rgb(cr / 5, cg / 5, cb / 5);
    let bg_near = Color::Rgb(cr / 9, cg / 9, cb / 9);
    let bg_mid = Color::Rgb(cr / 16, cg / 16, cb / 16);
    let bg_far = Color::Rgb(cr / 28, cg / 28, cb / 28);

    let total_w = area.width as usize;
    let mut lines: Vec<Line> = Vec::with_capacity(area.height as usize);

    for row in 0..area.height {
        let dy = row as f64 + 0.5 - cy;
        let abs_dy = dy.abs();
        let row_bg = if abs_dy < glow_r + 4.0 {
            bg_near
        } else if abs_dy < glow_r + 9.0 {
            bg_mid
        } else {
            bg_far
        };

        if abs_dy >= glow_r + 0.5 {
            lines.push(Line::from(Span::styled(
                " ".repeat(total_w),
                Style::default().bg(row_bg),
            )));
            continue;
        }

        let (fill_l, fill_r) = if abs_dy < base_r {
            let half = ((base_r * base_r - dy * dy).sqrt() * 2.0).round() as usize;
            let l = (cx as usize).saturating_sub(half / 2);
            let r = (l + half).min(total_w);
            (l, r)
        } else {
            (cx as usize, cx as usize)
        };

        let (glow_l, glow_r_col) = if abs_dy < glow_r {
            let half = ((glow_r * glow_r - dy * dy).sqrt() * 2.0).round() as usize;
            let l = (cx as usize).saturating_sub(half / 2);
            let r = (l + half).min(total_w);
            (l, r)
        } else {
            (fill_l, fill_r)
        };

        let mut spans: Vec<Span> = Vec::new();
        let bg = Style::default().bg(row_bg);

        if glow_l > 0 {
            spans.push(Span::styled(" ".repeat(glow_l), bg));
        }

        if fill_l > glow_l {
            let n = fill_l - glow_l;
            spans.push(Span::styled(
                "░".repeat(n),
                Style::default().fg(glow_color).bg(row_bg),
            ));
        }

        let fill_w = fill_r.saturating_sub(fill_l);
        if fill_w > 0 {
            let edge_n = (fill_w / 5).clamp(1, 4);
            if fill_w <= edge_n * 2 {
                spans.push(Span::styled(
                    "▓".repeat(fill_w),
                    Style::default().fg(edge_color).bg(row_bg),
                ));
            } else {
                let core_n = fill_w - edge_n * 2;
                spans.push(Span::styled(
                    "▓".repeat(edge_n),
                    Style::default().fg(edge_color).bg(row_bg),
                ));
                spans.push(Span::styled(
                    "█".repeat(core_n),
                    Style::default().fg(fill_color).bg(row_bg),
                ));
                spans.push(Span::styled(
                    "▓".repeat(edge_n),
                    Style::default().fg(edge_color).bg(row_bg),
                ));
            }
        }

        if glow_r_col > fill_r {
            let n = glow_r_col - fill_r;
            spans.push(Span::styled(
                "░".repeat(n),
                Style::default().fg(glow_color).bg(row_bg),
            ));
        }

        let rendered = glow_r_col.min(total_w);
        if rendered < total_w {
            spans.push(Span::styled(" ".repeat(total_w - rendered), bg));
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}
