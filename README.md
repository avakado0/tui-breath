# TUI Breath — Interactive Breathing Guide

A terminal-based breathing guide built in Rust. Guides you through breathing exercises with an animated glowing orb, smooth color transitions, and automatic session tracking.

> **Web port:** [breath4.life](https://breath4.life) — same engine, pixel-art / phosphor-CRT aesthetic in the browser. Source: [breath4life-web](https://github.com/avakado0/breath4life-web).

## Installation

```bash
cargo install tui_breath
```

<details>
<summary>Build from source</summary>

```bash
git clone https://github.com/avakado0/tui-breath
cd tui-breath
cargo build --release
./target/release/tui_breath
```
</details>

**Uninstall:** `cargo uninstall tui_breath`

![demo](demo.gif)

## Features

**Animated Visualization**
- Expanding/contracting glowing circle — grows on inhale, shrinks on exhale
- Cubic easing for organic, non-mechanical movement
- Three-layer atmospheric background that shifts color with each phase
- Soft edge rendering (`▓` rim → `█` core → `░` glow halo)

**Phase Animation**
- Smooth 800ms color crossfade between phases (Cyan → Yellow → Green)
- Typewriter phase label reveal on each transition
- Hold phase: full orb with brightness pulse at ~900ms rhythm
- All animations implemented in-house (Lerp, Typewriter, Pulse structs)

**Breathing Patterns**
- **4-7-8**: 4s inhale, 7s hold, 8s exhale
- **Box Breathing**: 4s each — inhale, hold, exhale, hold
- **Diaphragmatic**: 4s inhale, 6s exhale
- **Breath of Fire**: rapid 0.5s inhale, 0.5s exhale
- **Bhastrika**: forceful 1s inhale, 1s exhale
- **Stimulating Breath**: brisk 0.4s inhale, 0.4s exhale

**Customization**
- Session duration: 1–100 breathing cycles (1 unit = 1 complete cycle, shown in minutes)
- Breathing speed: 0.5×–2.0× (scales all phase durations)

- Audio beep on phase transitions via terminal bell (toggle with `b`, off by default)

**Session Tracking**
- Metrics: cycles, pauses, completion %, breathing rate
- Persistent JSON storage at `~/.local/share/tui_breath/`
- Auto-saves completed sessions
- Browsable history

## Quick Start

Minimum terminal size: **60×24**.

## Controls

| Screen | Key | Action |
|--------|-----|--------|
| Menu | `j/k` `↑/↓` | Navigate patterns |
| Menu | `Enter` | Select |
| Menu | `h` | History |
| Setup | `Tab` | Switch field (Duration / Breathing Speed) |
| Setup | `↑/↓` or `+/-` | Adjust value |
| Setup | `Esc` | Back to menu |
| Setup | `Enter` | Start session |
| Session | `p` / `Space` | Pause / Resume |
| Session | `e` / `Esc` | End early |
| Results | `s` | Save again, then return to menu |
| Results | `Enter` / `Esc` | Return to menu |
| History | `j/k` `↑/↓` | Navigate saved sessions |
| History | `Esc` | Back to menu |
| Any | `b` | Toggle beep |
| Any | `q` / `Ctrl-C` | Quit |

## Architecture

```
src/
├── main.rs              # 30 FPS event loop, SessionAnimator::tick()
├── app.rs               # State machine (Menu→Setup→Session→Results→History)
├── animator.rs          # SessionAnimator — animated color/label/pulse fields
├── audio.rs             # Terminal bell beeper for phase transitions
├── engine/
│   ├── breathing.rs     # BreathingEngine (Copy) — phase timing, progress
│   ├── patterns.rs      # Six pattern definitions, channels, visual fill rules
│   └── session.rs       # SessionManager — event log, metrics
├── storage/             # JSON persistence
└── ui/
    ├── menu.rs          # Pattern picker
    ├── setup.rs         # Duration / tempo cards and phase bar
    ├── session.rs       # Glowing circle renderer, three-zone background
    ├── results.rs       # Completion metrics and summary
    └── history.rs       # Saved session browser
```

**Animation pipeline:**
```
BreathingEngine::phase_progress()  →  cubic_in_out()  →  circle radius
                   ↓
           phase transition  →  SessionAnimator::set()  →  color crossfade
                                                        →  typewriter label
                                                        →  hold pulse
```

The engine owns timing. `animator.rs` owns visual interpolation.

## Performance

| Metric | Value |
|--------|-------|
| Refresh rate | 30 FPS (33ms ticks) |
| Binary size | ~8MB (release, LTO) |
| Memory | <10MB |
| CPU | <5% |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui 0.27` | TUI layout and rendering |
| `crossterm 0.27` | Terminal I/O |
| `tokio` | Async runtime |
| `serde_json` | Session persistence |
| `uuid`, `chrono`, `dirs`, `anyhow` | Utilities |

Animation approach inspired by [animate](https://github.com/vyfor/animate) by vyfor.

## Testing

```bash
cargo test
```

Covers phase progression, tempo scaling, completion detection, pause/resume.

## Session Storage

- **Linux/macOS**: `~/.local/share/tui_breath/sessions/`
- **Windows**: `%APPDATA%\Local\tui_breath\sessions\`

Index at `~/.local/share/tui_breath/index.json` for fast history loads.
