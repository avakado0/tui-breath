use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

use crate::app::{App, AppState, BodyMovementsState};
use crate::engine::patterns::PhaseStyle;

pub struct Step {
    pub label: &'static str,
    pub art: &'static str,
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
                art: "\
   crown up ──► ╭───╮
               (     )
                ╰───╯
                  │
                ──┼──  ← arms relaxed
                  │
                 ╱│╲
                ╱ │ ╲
               ╱  │  ╲
  ════════════╧═══════╧════════════
  ╔══════════════════════════════╗
  ║          sit tall            ║
  ╚══════════════════════════════╝
       │                 │
       └─────────────────┘
           feet flat",
                cue: "Sit tall, relax shoulders, feet flat on floor",
            },
            Step {
                label: "Grip & Rotate",
                art: "\
        eyes follow ──►
           ╭───╮  ↖
          (     ) ↗
           ╰───╯
        ↙╲    │
          ╲   │  ← spine
           ╲  │    twists
            ╲ │
             ╲│
              │  ╲
  ════════════╧═══════╧════════════
  ╔══════════════════════════════╗
  ║   grip chair back here  →   ║
  ╚══════════════════════════════╝
       │                 │
       └─────────────────┘
           feet flat",
                cue: "Grip chair back, exhale into the twist, hips stay square",
            },
            Step {
                label: "Deepen",
                art: "\
  hand behind head ──►
     ╭───╮ ←─┐
    (     )   │
     ╰───╯   hand
   ╲    │
    ╲   │
     ╲  │
      ╲ │
       ╲│
        │  ╲
  ════════════╧═══════╧════════════
  ╔══════════════════════════════╗
  ║      grip + pull  →         ║
  ╚══════════════════════════════╝
       │                 │
       └─────────────────┘
           feet flat",
                cue: "Each exhale → 2–3° deeper. Look over shoulder. No wrenching.",
            },
        ],
    },
    Movement {
        name: "Hip Flexor Stretch",
        steps: [
            Step {
                label: "Edge of Seat",
                art: "\
        ╭───╮
       (     )
        ╰───╯
          │
        ──┼──
          │
         ╱│╲
        ╱ │ ╲
       ╱  │  ╲
  ════╧═══════════╧════
  ╔═════════════════╗
  ║  sit at EDGE   ║
  ╚═════════════════╝
       │         │
       └─────────┘
      both feet flat",
                cue: "Sit at the edge, back straight, both feet flat",
            },
            Step {
                label: "Slide Leg Back",
                art: "\
        ╭───╮
       (     )
        ╰───╯
          │
        ──┼──
          │
         ╱│╲
        ╱ │ ╲─────────────────► foot
       ╱  │   ╲   leg slides back
  ════╧══════════╧════
  ╔═════════════════╗
  ║   chair edge   ║
  ╚═════════════════╝
       │         │
       └─────────┘
      both feet flat",
                cue: "Slide one leg back, foot flat on floor, feel hip crease",
            },
            Step {
                label: "Tilt & Lean",
                art: "\
       ↙╭───╮
       ↙(     )
        ╰───╯
       ↙  │
        ──┼──
       ↙ ╱│╲
       ╱ │  ╲──────────────► foot
      ╱  │   ╲   tuck tailbone
  ════╧══════════╧════
  ╔═════════════════╗
  ║   chair edge   ║
  ╚═════════════════╝
       │         │
       └─────────┘
      both feet flat",
                cue: "Hinge at hips, squeeze back glute, feel front of hip pull",
            },
        ],
    },
    Movement {
        name: "Neck Lateral Stretch",
        steps: [
            Step {
                label: "Relax & Sit",
                art: "\
  crown lifts ──►  ╭─────╮
                  ( crown )
                   ╰─────╯
                       │
                   ────┼────
                  ╱    │    ╲
                 ╱     │     ╲
                ╱      │      ╲
  ═════════════╧═══════════════╧══
  ╔═══════════════════════════╗
  ║                           ║
  ╚═══════════════════════════╝
        │               │
        └───────────────┘",
                cue: "Sit tall, relax shoulders, eyes forward, breathe easy",
            },
            Step {
                label: "Ear to Shoulder",
                art: "\
                  ╭─────╮
                 (  tips )──────►
                  ╰─────╯
                ╱    │
               ╱  ───┼───
              ╱  ╱   │   ╲
             ╱  ╱    │    ╲
            ╱  ╱     │     ╲
  ═════════════╧═══════════════╧══
  ╔═══════════════════════════╗
  ║                           ║
  ╚═══════════════════════════╝
        │               │
        └───────────────┘",
                cue: "Drop ear toward shoulder. Shoulder stays pinned DOWN — don't shrug.",
            },
            Step {
                label: "Hand Assist",
                art: "\
            ←─(hand)
                  ╭─────╮
                 (  tips )──────►
                  ╰─────╯
              ╱       │
             ╱    ────┼────
            ╱  ╱     │    ╲
           ╱  ╱      │     ╲
          ╱  ╱       │      ╲
  ═════════════╧═══════════════╧══
  ╔═══════════════════════════╗
  ║                           ║
  ╚═══════════════════════════╝
        │               │
        └───────────────┘",
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
    let beep_status = if app.beeper.is_enabled() { "🔊" } else { "🔇" };
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

    // Art
    let art_height = inner.height.saturating_sub(4);
    let art_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: art_height,
    };
    let art = Paragraph::new(step.art)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    f.render_widget(art, art_area);

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
        (*anim.color_r as u8, *anim.color_g as u8, *anim.color_b as u8)
    } else {
        match phase.style {
            PhaseStyle::Rising => (0, 255, 255),
            PhaseStyle::Steady => (255, 230, 0),
            PhaseStyle::Falling => (0, 220, 100),
        }
    };
    let phase_color = Color::Rgb(cr, cg, cb);

    use ratatui::text::{Line, Text};
    use crate::engine::patterns::Channel;
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
        Line::from(channel_str)
            .style(Style::default().fg(Color::Rgb(cr / 2, cg / 2, cb / 2))),
        Line::from(""),
        Line::from(format!("Cycle {}  ·  {:.0}%", engine.cycle_count, engine.completion_percent()))
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
