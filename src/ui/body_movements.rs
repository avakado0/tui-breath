use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

use crate::app::{App, AppState, BodyMovementsState};
use crate::engine::patterns::PhaseStyle;

pub struct Step {
    pub label: &'static str,
    pub cue: &'static str,
}

pub struct Movement {
    pub name: &'static str,
    pub steps: [Step; 3],
}

pub static MOVEMENTS: [Movement; 3] = [
    Movement {
        name: "Seated Spinal Twist",
        steps: [
            Step {
                label: "Sit Tall",
                cue: "Sit tall, relax shoulders, feet flat on floor",
            },
            Step {
                label: "Grip & Rotate",
                cue: "Grip chair back, exhale into the twist, hips stay square",
            },
            Step {
                label: "Deepen",
                cue: "Each exhale → 2–3° deeper. Look over shoulder. No wrenching.",
            },
        ],
    },
    Movement {
        name: "Hip Flexor Stretch",
        steps: [
            Step {
                label: "Edge of Seat",
                cue: "Sit at the edge, back straight, both feet flat",
            },
            Step {
                label: "Slide Leg Back",
                cue: "Slide one leg back, foot flat on floor, feel hip crease",
            },
            Step {
                label: "Tilt & Lean",
                cue: "Hinge at hips, squeeze back glute, feel front of hip pull",
            },
        ],
    },
    Movement {
        name: "Neck Lateral Stretch",
        steps: [
            Step {
                label: "Relax & Sit",
                cue: "Sit tall, relax shoulders, eyes forward, breathe easy",
            },
            Step {
                label: "Ear to Shoulder",
                cue: "Drop ear toward shoulder. Shoulder stays pinned DOWN — don't shrug.",
            },
            Step {
                label: "Hand Assist",
                cue: "Opposite hand adds gentle pressure. Chin forward, face doesn't rotate.",
            },
        ],
    },
];

pub fn draw(f: &mut Frame, app: &App) {
    let AppState::BodyMovements(bm) = &app.state else {
        return;
    };

    let area = f.size();

    let outer = Block::default()
        .title(" Body Movements ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(outer, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Footer row
    let footer_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };
    let beep_status = if app.beeper.is_enabled() {
        "🔊"
    } else {
        "🔇"
    };
    let footer = Paragraph::new(format!(
        "[1] Spinal Twist  [2] Hip Flexor  [3] Neck  [m/Esc] Back  [b] {} Beep  [q] Quit",
        beep_status
    ))
    .alignment(Alignment::Center)
    .style(Style::default().dim());
    f.render_widget(footer, footer_area);

    let content_height = inner.height.saturating_sub(1);

    // Split: left ~50%, right ~50%
    let right_width = (inner.width / 2).max(30);
    let left_width = inner.width.saturating_sub(right_width);

    let left_area = Rect {
        x: inner.x,
        y: inner.y,
        width: left_width,
        height: content_height,
    };
    let right_area = Rect {
        x: inner.x + left_width,
        y: inner.y,
        width: right_width,
        height: content_height,
    };

    render_movement_panel(f, left_area, bm);
    render_breathing_panel(f, right_area, app);
}

fn render_movement_panel(f: &mut Frame, area: Rect, bm: &BodyMovementsState) {
    let movement = &MOVEMENTS[bm.current_movement];
    let step = &movement.steps[bm.current_step];

    let block = Block::default()
        .title(format!(
            " {} · Step {}/3: {} ",
            movement.name,
            bm.current_step + 1,
            step.label
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let view_label = movement_view_label(bm.current_movement);
    let view_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    let view = Paragraph::new(view_label)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).bold());
    f.render_widget(view, view_area);

    // Body art uses 2x3 sextant cells so the poses read as silhouettes, not text diagrams.
    let art_height = inner.height.saturating_sub(5);
    let art_area = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: art_height,
    };
    render_pose_art(f, art_area, bm.current_movement, bm.current_step);

    // Cue line
    let cue_area = Rect {
        x: inner.x,
        y: inner.y + art_height,
        width: inner.width,
        height: 1,
    };
    let cue = Paragraph::new(step.cue)
        .style(Style::default().fg(Color::Green).italic())
        .alignment(Alignment::Center);
    f.render_widget(cue, cue_area);

    // Step progress bar — synced to breath phase
    let bar_area = Rect {
        x: inner.x,
        y: inner.y + art_height + 1,
        width: inner.width,
        height: 1,
    };
    let engine = &bm.session_state.manager.engine;
    let phase_progress = engine.phase_progress();
    let phase_remaining = engine.phase_remaining();
    let gauge = Gauge::default()
        .label(format!("{:.0}s left", phase_remaining))
        .ratio(phase_progress.clamp(0.0, 1.0))
        .gauge_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
    f.render_widget(gauge, bar_area);

    // Movement selector hint
    let selector_area = Rect {
        x: inner.x,
        y: inner.y + art_height + 2,
        width: inner.width,
        height: 1,
    };
    let selector_items: Vec<&str> = MOVEMENTS
        .iter()
        .enumerate()
        .map(|(i, m)| {
            // just show the number and short name
            let _ = m;
            match i {
                0 => "[1] Twist",
                1 => "[2] Hip",
                2 => "[3] Neck",
                _ => "",
            }
        })
        .collect();
    let selector_text = selector_items
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if i == bm.current_movement {
                format!("▶ {} ◀", s)
            } else {
                s.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("  ");
    let selector = Paragraph::new(selector_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(selector, selector_area);
}

fn movement_view_label(movement: usize) -> &'static str {
    match movement {
        0 => "View: 3/4 front twist - chair back is on the right",
        1 => "View: side profile - body faces right, stretch leg reaches back left",
        2 => "View: front - shoulders face you",
        _ => "View: front",
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PixelColor(u8, u8, u8);

const SKIN: PixelColor = PixelColor(210, 155, 95);
const HAIR: PixelColor = PixelColor(25, 18, 18);
const SHIRT: PixelColor = PixelColor(75, 115, 175);
const PANTS: PixelColor = PixelColor(48, 58, 82);
const CHAIR: PixelColor = PixelColor(118, 87, 52);
const CHAIR_DARK: PixelColor = PixelColor(72, 48, 30);
const FLOOR: PixelColor = PixelColor(52, 52, 64);
const AMBER: PixelColor = PixelColor(255, 198, 70);

const CANVAS_W: usize = 50;
const CANVAS_H: usize = 45;
const CELL_W: usize = CANVAS_W / 2;
const CELL_H: usize = CANVAS_H / 3;

struct PixelCanvas {
    pixels: [[Option<PixelColor>; CANVAS_W]; CANVAS_H],
}

impl PixelCanvas {
    fn new() -> Self {
        Self {
            pixels: [[None; CANVAS_W]; CANVAS_H],
        }
    }

    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: PixelColor) {
        let x0 = x.max(0) as usize;
        let y0 = y.max(0) as usize;
        let x1 = (x + w).clamp(0, CANVAS_W as i32) as usize;
        let y1 = (y + h).clamp(0, CANVAS_H as i32) as usize;

        for row in y0..y1 {
            for col in x0..x1 {
                self.pixels[row][col] = Some(color);
            }
        }
    }

    fn circle(&mut self, cx: i32, cy: i32, radius: i32, color: PixelColor) {
        for row in (cy - radius).max(0)..=(cy + radius).min(CANVAS_H as i32 - 1) {
            for col in (cx - radius).max(0)..=(cx + radius).min(CANVAS_W as i32 - 1) {
                let dx = col - cx;
                let dy = row - cy;
                if dx * dx + dy * dy <= radius * radius {
                    self.pixels[row as usize][col as usize] = Some(color);
                }
            }
        }
    }

    fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: PixelColor) {
        if rx <= 0 || ry <= 0 {
            return;
        }

        for row in (cy - ry).max(0)..=(cy + ry).min(CANVAS_H as i32 - 1) {
            for col in (cx - rx).max(0)..=(cx + rx).min(CANVAS_W as i32 - 1) {
                let dx = (col - cx) as f32 / rx as f32;
                let dy = (row - cy) as f32 / ry as f32;
                if dx * dx + dy * dy <= 1.0 {
                    self.pixels[row as usize][col as usize] = Some(color);
                }
            }
        }
    }

    fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, thickness: i32, color: PixelColor) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let steps = (((dx * dx + dy * dy) as f32).sqrt().ceil() as i32).max(1);

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = (x0 as f32 + dx as f32 * t).round() as i32;
            let y = (y0 as f32 + dy as f32 * t).round() as i32;
            self.rect(
                x - thickness / 2,
                y - thickness / 2,
                thickness,
                thickness,
                color,
            );
        }
    }

    fn to_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::with_capacity(CELL_H);

        for row in (0..CANVAS_H).step_by(3) {
            let mut spans = Vec::with_capacity(CELL_W);
            for col in (0..CANVAS_W).step_by(2) {
                let px = [
                    self.pixels[row][col],
                    self.pixels[row][col + 1],
                    self.pixels[row + 1][col],
                    self.pixels[row + 1][col + 1],
                    self.pixels[row + 2][col],
                    self.pixels[row + 2][col + 1],
                ];

                spans.push(sextant_span(px));
            }
            lines.push(Line::from(spans));
        }

        lines
    }
}

fn render_pose_art(f: &mut Frame, area: Rect, movement: usize, step: usize) {
    let lines = pose_canvas(movement, step).to_lines();
    let text_width = CELL_W as u16;
    let text_height = CELL_H as u16;
    let x = area.x + area.width.saturating_sub(text_width) / 2;
    let y = area.y + area.height.saturating_sub(text_height) / 2;
    let pose_area = Rect {
        x,
        y,
        width: text_width.min(area.width),
        height: text_height.min(area.height),
    };

    f.render_widget(Paragraph::new(lines), pose_area);
}

fn sextant_span(px: [Option<PixelColor>; 6]) -> Span<'static> {
    let Some(fg) = dominant_color(&px, None) else {
        return Span::raw(" ");
    };
    let bg = dominant_color(&px, Some(fg));
    let bits = px.map(|color| {
        if bg.is_some() {
            color == Some(fg)
        } else {
            color.is_some()
        }
    });
    let ch = sextant_char(bits);
    let mut style = Style::default().fg(to_ratatui_color(fg));
    if let Some(bg) = bg {
        style = style.bg(to_ratatui_color(bg));
    }

    Span::styled(ch.to_string(), style)
}

fn dominant_color(px: &[Option<PixelColor>; 6], exclude: Option<PixelColor>) -> Option<PixelColor> {
    let mut counts: Vec<(PixelColor, usize)> = Vec::new();

    for color in px.iter().flatten().copied() {
        if Some(color) == exclude {
            continue;
        }
        if let Some((_, count)) = counts.iter_mut().find(|(existing, _)| *existing == color) {
            *count += 1;
        } else {
            counts.push((color, 1));
        }
    }

    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(color, _)| color)
}

fn sextant_char(bits: [bool; 6]) -> char {
    let value = bits
        .iter()
        .enumerate()
        .fold(0u8, |acc, (i, bit)| if *bit { acc | (1 << i) } else { acc });

    match value {
        0 => ' ',
        63 => '█',
        value => char::from_u32(0x1FB00 + value as u32 - 1).unwrap_or('█'),
    }
}

fn to_ratatui_color(color: PixelColor) -> Color {
    Color::Rgb(color.0, color.1, color.2)
}

fn pose_canvas(movement: usize, step: usize) -> PixelCanvas {
    match (movement, step) {
        (0, 0) => seated_front(false),
        (0, 1) => spinal_twist_rotate(),
        (0, 2) => spinal_twist_deepen(),
        (1, 0) => seated_side(0),
        (1, 1) => seated_side(14),
        (1, 2) => hip_flexor_lean(),
        (2, 0) => seated_front(true),
        (2, 1) => neck_tilt(),
        (2, 2) => neck_hand_assist(),
        _ => seated_front(false),
    }
}

fn seated_front(crown: bool) -> PixelCanvas {
    let mut g = PixelCanvas::new();
    draw_front_base(&mut g, 25, 0, crown);
    g
}

fn draw_front_base(g: &mut PixelCanvas, head_x: i32, head_y: i32, crown: bool) {
    draw_chair_front_backrest(g);
    g.ellipse(head_x, 6 + head_y, 6, 4, HAIR);
    g.ellipse(head_x, 8 + head_y, 5, 6, SKIN);
    g.rect(head_x - 2, 14 + head_y, 5, 4, SKIN);
    g.rect(14, 18, 22, 13, SHIRT);
    g.rect(4, 18, 10, 4, SHIRT);
    g.rect(4, 22, 4, 9, SHIRT);
    g.rect(36, 18, 10, 4, SHIRT);
    g.rect(42, 22, 4, 9, SHIRT);
    g.rect(14, 31, 22, 8, PANTS);
    draw_chair_front(g);

    if crown {
        g.rect(head_x - 4, 2 + head_y, 8, 2, AMBER);
        g.ellipse(head_x, 3 + head_y, 5, 2, AMBER);
    }
}

fn draw_chair_front(g: &mut PixelCanvas) {
    draw_chair_front_seat(g);
}

fn draw_chair_front_backrest(g: &mut PixelCanvas) {
    g.rect(8, 24, 4, 17, CHAIR_DARK);
    g.rect(38, 24, 4, 17, CHAIR_DARK);
    g.rect(8, 24, 34, 4, CHAIR_DARK);
    g.rect(11, 27, 28, 3, CHAIR);
}

fn draw_chair_front_seat(g: &mut PixelCanvas) {
    g.rect(5, 36, 40, 3, CHAIR_DARK);
    g.rect(7, 38, 36, 4, CHAIR);
    g.rect(7, 41, 4, 4, CHAIR_DARK);
    g.rect(39, 41, 4, 4, CHAIR_DARK);
    g.rect(0, 43, CANVAS_W as i32, 2, FLOOR);
}

fn spinal_twist_rotate() -> PixelCanvas {
    let mut g = PixelCanvas::new();
    draw_chair_front_backrest(&mut g);
    g.ellipse(27, 6, 6, 4, HAIR);
    g.ellipse(27, 8, 5, 6, SKIN);
    g.rect(25, 14, 5, 4, SKIN);
    g.rect(14, 18, 22, 13, SHIRT);
    g.line(4, 20, 39, 25, 4, AMBER);
    draw_grip_hand(&mut g, 38, 23);
    g.rect(36, 18, 6, 3, PANTS);
    g.rect(14, 31, 22, 8, PANTS);
    draw_chair_front(&mut g);
    g
}

fn draw_grip_hand(g: &mut PixelCanvas, x: i32, y: i32) {
    g.rect(x, y, 4, 7, AMBER);
    g.rect(x + 4, y + 2, 2, 3, AMBER);
}

fn draw_relaxed_hand(g: &mut PixelCanvas, x: i32, y: i32) {
    g.rect(x, y, 5, 4, AMBER);
    g.rect(x + 1, y - 2, 3, 2, AMBER);
}

fn spinal_twist_deepen() -> PixelCanvas {
    let mut g = PixelCanvas::new();
    draw_chair_front_backrest(&mut g);
    g.ellipse(28, 6, 6, 4, HAIR);
    g.ellipse(28, 8, 5, 6, SKIN);
    g.rect(26, 14, 5, 4, SKIN);
    g.rect(14, 18, 22, 13, SHIRT);
    g.rect(4, 18, 8, 4, SHIRT);
    g.rect(4, 10, 4, 9, AMBER);
    draw_relaxed_hand(&mut g, 4, 7);
    g.line(36, 20, 39, 25, 4, AMBER);
    draw_grip_hand(&mut g, 38, 23);
    g.rect(14, 31, 22, 8, PANTS);
    draw_chair_front(&mut g);
    g
}

fn seated_side(back_leg: i32) -> PixelCanvas {
    let mut g = PixelCanvas::new();
    draw_chair_side(&mut g);

    // Side profile: the face, chest, front knee, and foot all point right.
    g.rect(10, 15, 13, 21, SHIRT);
    g.rect(17, 9, 5, 7, SKIN);
    g.ellipse(22, 4, 6, 5, HAIR);
    g.ellipse(25, 6, 7, 7, SKIN);
    g.rect(30, 6, 4, 2, SKIN);
    g.circle(29, 5, 1, HAIR);
    g.rect(21, 18, 9, 4, SHIRT);
    g.rect(27, 22, 4, 9, SHIRT);
    g.rect(10, 36, 25, 5, PANTS);
    g.rect(30, 39, 5, 6, PANTS);
    g.rect(30, 44, 10, 1, SKIN);

    if back_leg > 0 {
        g.rect(10 - back_leg, 36, back_leg, 5, AMBER);
        g.rect(10 - back_leg, 40, 5, 4, AMBER);
    }

    g
}

fn draw_chair_side(g: &mut PixelCanvas) {
    g.rect(2, 13, 5, 28, CHAIR_DARK);
    g.rect(5, 16, 5, 19, CHAIR);
    g.rect(2, 34, 31, 4, CHAIR_DARK);
    g.rect(6, 36, 29, 4, CHAIR);
    g.rect(4, 39, 4, 6, CHAIR_DARK);
    g.rect(30, 39, 4, 6, CHAIR_DARK);
    g.rect(0, 43, CANVAS_W as i32, 2, FLOOR);
}

fn hip_flexor_lean() -> PixelCanvas {
    let mut g = PixelCanvas::new();
    draw_chair_side(&mut g);
    g.line(12, 35, 26, 14, 14, SHIRT);
    g.ellipse(29, 7, 6, 5, HAIR);
    g.ellipse(32, 10, 7, 7, SKIN);
    g.rect(37, 10, 4, 2, SKIN);
    g.circle(36, 8, 1, HAIR);
    g.line(20, 15, 26, 11, 4, SKIN);
    g.rect(0, 36, 12, 5, AMBER);
    g.rect(0, 40, 5, 4, AMBER);
    g.rect(10, 36, 24, 5, PANTS);
    g.rect(10, 40, 6, 5, PANTS);
    g.rect(10, 44, 10, 1, SKIN);
    g
}

fn neck_tilt() -> PixelCanvas {
    let mut g = PixelCanvas::new();
    draw_simple_front_body(&mut g);
    draw_tilted_head_front(&mut g);
    g.rect(36, 20, 8, 4, AMBER);
    g.rect(41, 24, 4, 9, AMBER);
    g
}

fn neck_hand_assist() -> PixelCanvas {
    let mut g = PixelCanvas::new();
    draw_simple_front_body(&mut g);
    draw_tilted_head_front(&mut g);
    g
}

fn draw_simple_front_body(g: &mut PixelCanvas) {
    g.rect(14, 20, 22, 14, SHIRT);
    g.rect(6, 20, 8, 4, SHIRT);
    g.rect(4, 24, 4, 9, SHIRT);
    g.rect(36, 20, 8, 4, SHIRT);
    g.rect(42, 24, 4, 9, SHIRT);
    g.rect(14, 34, 22, 8, PANTS);
    g.rect(0, 43, CANVAS_W as i32, 2, FLOOR);
}

fn draw_tilted_head_front(g: &mut PixelCanvas) {
    g.line(25, 17, 30, 18, 4, SKIN);
    g.ellipse(32, 9, 6, 4, HAIR);
    g.ellipse(32, 12, 6, 6, SKIN);
}

fn render_breathing_panel(f: &mut Frame, area: Rect, app: &App) {
    let AppState::BodyMovements(bm) = &app.state else {
        return;
    };

    let engine = &bm.session_state.manager.engine;
    let phase = engine.current_phase();

    let block = Block::default()
        .title(" Breathing ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).dim());
    f.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Circle animation — takes up most of the panel
    let stats_rows = 5u16;
    let circle_height = inner.height.saturating_sub(stats_rows).max(4);
    let circle_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: circle_height,
    };
    crate::ui::session::render_breathing_circle_for(
        f,
        circle_area,
        engine,
        app.session_animator.as_ref(),
    );

    // Phase label + remaining
    let (cr, cg, cb) = if let Some(anim) = &app.session_animator {
        (
            *anim.color_r as u8,
            *anim.color_g as u8,
            *anim.color_b as u8,
        )
    } else {
        match phase.style {
            PhaseStyle::Rising => (0, 255, 255),
            PhaseStyle::Steady => (255, 230, 0),
            PhaseStyle::Falling => (0, 220, 100),
        }
    };
    let phase_color = Color::Rgb(cr, cg, cb);

    use crate::engine::patterns::Channel;
    use ratatui::text::{Line, Text};
    let channel_str = match phase.channel {
        Some(Channel::Nose) => "░ nose ░",
        Some(Channel::Mouth) => "░ mouth ░",
        None => "",
    };
    let remaining = (engine.phase_remaining() * 10.0).ceil() / 10.0;
    let stats_text = Text::from(vec![
        Line::from(format!("*** {} ***", phase.name))
            .style(Style::default().fg(phase_color).bold()),
        Line::from(format!("{remaining:.1}s remaining"))
            .style(Style::default().fg(phase_color).bold()),
        Line::from(channel_str).style(Style::default().fg(Color::Rgb(cr / 2, cg / 2, cb / 2))),
        Line::from(""),
        Line::from(format!(
            "Cycle {}  ·  {:.0}%",
            engine.cycle_count,
            engine.completion_percent()
        ))
        .style(Style::default().dim()),
    ]);
    let stats_area = Rect {
        x: inner.x,
        y: inner.y + circle_height,
        width: inner.width,
        height: stats_rows,
    };
    f.render_widget(
        Paragraph::new(stats_text).alignment(Alignment::Center),
        stats_area,
    );
}
