use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

use crate::app::{App, AppState};
use crate::engine::patterns::PhaseStyle;

pub fn draw(f: &mut Frame, app: &App) {
    let AppState::Session(session_state) = &app.state else {
        return;
    };

    let engine = &session_state.manager.engine;
    let pattern = engine.pattern;
    let area = f.size();

    let title = if engine.is_paused {
        format!("{} [PAUSED]", pattern.display_name)
    } else {
        pattern.display_name.to_string()
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

    // Give circle as much vertical space as possible (reserve 7 rows for label/stats/gauge)
    let animation_height = inner.height.saturating_sub(7).max(5);
    let animation_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: animation_height,
    };

    render_breathing_circle(f, animation_area, app);

    // Phase label and countdown
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

    let phase_text = format!("*** {} ***\n{:.1}s remaining", phase_label, remaining);
    let phase_para = Paragraph::new(phase_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(phase_color).bold());

    let phase_area = Rect {
        x: inner.x,
        y: inner.y + animation_height + 1,
        width: inner.width,
        height: 3,
    };
    f.render_widget(phase_para, phase_area);

    // Stats line
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
        y: inner.y + animation_height + 4,
        width: inner.width,
        height: 1,
    };
    f.render_widget(stats_para, stats_area);

    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::Rgb(cr / 2, cg / 2, cb / 2)))
        .percent(engine.completion_percent() as u16);

    let gauge_area = Rect {
        x: inner.x,
        y: inner.y + animation_height + 5,
        width: inner.width,
        height: 2,
    };
    f.render_widget(gauge, gauge_area);

    let beep_status = if app.beeper.is_enabled() { "🔊" } else { "🔇" };
    let footer = if engine.is_paused {
        format!("[p] Resume  [e] End  [b] {} Beep  [q] Quit", beep_status)
    } else {
        format!("[p] Pause  [e] End  [b] {} Beep  [q] Quit", beep_status)
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

fn get_anim_color(app: &App, style: &PhaseStyle) -> (u8, u8, u8) {
    if let Some(anim) = &app.session_animator {
        (*anim.color_r as u8, *anim.color_g as u8, *anim.color_b as u8)
    } else {
        match style {
            PhaseStyle::Rising => (0, 255, 255),
            PhaseStyle::Steady => (255, 230, 0),
            PhaseStyle::Falling => (0, 220, 100),
        }
    }
}

fn render_breathing_circle(f: &mut Frame, area: Rect, app: &App) {
    let AppState::Session(session_state) = &app.state else {
        return;
    };
    let engine = &session_state.manager.engine;
    let phase = engine.current_phase();

    let progress = crate::animator::cubic_in_out(engine.phase_progress());

    let h = area.height as f64;
    let w = area.width as f64;
    let cy = h / 2.0;
    let cx = w / 2.0;
    // Terminal chars are ~2:1 (tall:wide in pixels), so x-extent = radius * 2
    // Max radius limited by height (cy) and by width (cx/2 after aspect correction)
    let max_r = cy.min(cx / 2.0) * 0.95;

    let ratio = crate::engine::patterns::fill_ratio(
        engine.pattern.phases,
        engine.current_phase_idx,
        progress,
    );
    let base_r = ratio * max_r;

    // Glow extends 2 char-rows beyond the fill boundary
    let glow_r = base_r + 2.0;

    // Animated color (with hold-phase brightness pulse on color)
    let (cr, cg, cb) = if let Some(anim) = &app.session_animator {
        let (r, g, b) = (*anim.color_r, *anim.color_g, *anim.color_b);
        if matches!(phase.style, PhaseStyle::Steady) {
            let pulse = *anim.hold_pulse; // oscillates 0.65 ↔ 1.0
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

    let fill_color = Color::Rgb(cr, cg, cb);
    let edge_color = Color::Rgb(
        (cr as u16 * 2 / 3) as u8,
        (cg as u16 * 2 / 3) as u8,
        (cb as u16 * 2 / 3) as u8,
    );
    let glow_color = Color::Rgb(cr / 5, cg / 5, cb / 5);

    // Three-zone background: dark tints of phase color, fading away from circle
    let bg_near   = Color::Rgb(cr / 9,  cg / 9,  cb / 9);   // close to orb
    let bg_mid    = Color::Rgb(cr / 16, cg / 16, cb / 16);   // medium
    let bg_far    = Color::Rgb(cr / 28, cg / 28, cb / 28);   // far corners

    let total_w = area.width as usize;
    let mut lines: Vec<Line> = Vec::with_capacity(area.height as usize);

    for row in 0..area.height {
        let dy = row as f64 + 0.5 - cy;
        let abs_dy = dy.abs();

        // Pick background tier based on row distance from circle edge
        let row_bg = if abs_dy < glow_r + 4.0 { bg_near }
                     else if abs_dy < glow_r + 9.0 { bg_mid }
                     else { bg_far };

        if abs_dy >= glow_r + 0.5 {
            // Outside glow: fill entire row with background color
            lines.push(Line::from(Span::styled(
                " ".repeat(total_w),
                Style::default().bg(row_bg),
            )));
            continue;
        }

        // Fill boundary: x-columns (aspect-corrected: x-extent = sqrt(r²-dy²) * 2)
        let (fill_l, fill_r) = if abs_dy < base_r {
            let half = ((base_r * base_r - dy * dy).sqrt() * 2.0).round() as usize;
            let l = (cx as usize).saturating_sub(half / 2);
            let r = (l + half).min(total_w);
            (l, r)
        } else {
            (cx as usize, cx as usize)
        };

        // Glow boundary
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

        // Leading space with background
        if glow_l > 0 {
            spans.push(Span::styled(" ".repeat(glow_l), bg));
        }

        // Left glow halo
        if fill_l > glow_l {
            let n = fill_l - glow_l;
            spans.push(Span::styled(
                "░".repeat(n),
                Style::default().fg(glow_color).bg(row_bg),
            ));
        }

        // Circle body with soft edge
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

        // Right glow halo
        if glow_r_col > fill_r {
            let n = glow_r_col - fill_r;
            spans.push(Span::styled(
                "░".repeat(n),
                Style::default().fg(glow_color).bg(row_bg),
            ));
        }

        // Trailing space with background
        let rendered = glow_r_col.min(total_w);
        if rendered < total_w {
            spans.push(Span::styled(" ".repeat(total_w - rendered), bg));
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}
