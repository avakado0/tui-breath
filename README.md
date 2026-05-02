# TUI Breath — Interactive Breathing Guide

A terminal-based breathing guide built in Rust. Guides you through breathing exercises with an animated glowing orb, smooth color transitions, and automatic session tracking.

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
- All animations driven by the [animate](https://github.com/vyfor/animate) library

**Breathing Patterns**
- **4-7-8**: 4s inhale, 7s hold, 8s exhale
- **Box Breathing**: 4s each — inhale, hold, exhale, hold
- **Diaphragmatic**: 4s inhale, 6s exhale

**Customization**
- Session duration: 1–60 minutes
- Tempo multiplier: 0.5×–2.0× (scales all phase durations)
- Audio beep on phase transitions (toggle with `b`)

**Session Tracking**
- Metrics: cycles, pauses, completion %, breathing rate
- Persistent JSON storage at `~/.local/share/tui_breath/`
- Browsable history

## Quick Start

```bash
cargo build --release
cargo run
```

Minimum terminal size: **60×24**.

## Controls

| Screen | Key | Action |
|--------|-----|--------|
| Menu | `j/k` `↑/↓` | Navigate patterns |
| Menu | `Enter` | Select |
| Menu | `h` | History |
| Setup | `Tab` | Switch field (Duration / Tempo) |
| Setup | `+/-` | Adjust value |
| Setup | `Enter` | Start session |
| Session | `p` / `Space` | Pause / Resume |
| Session | `e` / `Esc` | End early |
| Results | `s` | Save session |
| Any | `b` | Toggle beep |
| Any | `q` | Quit |

## Architecture

```
src/
├── main.rs              # 30 FPS event loop, animate::tick()
├── app.rs               # State machine (Menu→Setup→Session→Results→History)
├── animator.rs          # SessionAnimator — animated color/label/pulse fields
├── engine/
│   ├── breathing.rs     # BreathingEngine (Copy) — phase timing, progress
│   ├── patterns.rs      # Pattern definitions, PhaseStyle (Rising/Steady/Falling)
│   └── session.rs       # SessionManager — event log, metrics
├── storage/             # JSON persistence
└── ui/
    └── session.rs       # Glowing circle renderer, three-zone background
```

**Animation pipeline:**
```
BreathingEngine::phase_progress()  →  cubic_in_out()  →  circle radius
                   ↓
           phase transition  →  SessionAnimator::set()  →  color crossfade
                                                        →  typewriter label
                                                        →  hold pulse
```

The engine owns timing. `animate` owns visual interpolation.

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
| `animate` | Easing, color crossfade, typewriter animation |
| `serde_json` | Session persistence |
| `uuid`, `chrono`, `dirs`, `anyhow` | Utilities |

## Testing

```bash
cargo test
```

Covers phase progression, tempo scaling, completion detection, pause/resume.

## Session Storage

- **Linux/macOS**: `~/.local/share/tui_breath/sessions/`
- **Windows**: `%APPDATA%\Local\tui_breath\sessions\`

Index at `~/.local/share/tui_breath/index.json` for fast history loads.
