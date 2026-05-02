# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Commands

**Build & Run:**
```bash
cargo build              # Debug build (faster compile, slower runtime)
cargo build --release   # Release build (optimized, ~8MB, LTO enabled)
cargo run              # Run debug binary with output
cargo test             # Run all tests
cargo test --lib      # Run only library tests
```

**Run Specific Test:**
```bash
cargo test breathing::tests::test_phase_progression
```

**Debug:**
```bash
RUST_BACKTRACE=1 cargo run  # Run with backtrace on panic
```

## High-Level Architecture

### State Machine Flow (AppState)

The application follows a strict state machine pattern defined in `src/app.rs`:

```
Menu (pattern selection)
  ↓ [Enter]
Setup (duration + tempo config)
  ↓ [Enter]
Session (guided breathing with animation)
  ↓ [session ends or 'e' pressed]
Results (metrics display)
  ↓ ['s' = save session, Enter = return to menu]
History (browse past sessions)
  ↓ [Esc]
Menu
```

**Key file:** `src/app.rs` defines `AppState` enum with type-safe state variants and transitions. This prevents invalid state combinations at compile time.

### Event Loop & Timing (main.rs)

- **Single-threaded async runtime** with `tokio::main(flavor = "current_thread")`
- **100ms tick rate** = 10 FPS animation (standard for TUI apps)
- **Tick-driven updates**: Every component updates on `Event::Tick`, not continuous polling
- Terminal setup/cleanup guaranteed via scope guards in `main.rs`

### Core Modules

**Engine (src/engine/)**
- `breathing.rs`: Pure computation `BreathingEngine` (Copy struct). No I/O. Tracks:
  - Phase progression (`current_phase_idx`)
  - Phase elapsed time and progress ratio (`phase_elapsed_secs` / duration)
  - Cycle count, total elapsed time
  - Pause state and count
  - **Key method:** `tick(delta_secs)` advances timer, auto-transitions phases
  - **Tempo scaling:** `duration / tempo` (2.0x = half duration, 0.5x = double)
- `patterns.rs`: Static `Pattern` references (3 built-in: 4-7-8, Box, Diaphragmatic)
  - Each pattern has phases with durations and "styles" (Rising/Steady/Falling for animation)
- `session.rs`: `SessionManager` wraps engine + tracks events/metrics

**Storage (src/storage/)**
- `schema.rs`: Serde-serializable structures for JSON sessions
- `store.rs`: File I/O (sessions saved to `~/.local/share/tui_breath/sessions/`)

**UI (src/ui/)**
- State-to-screen dispatcher in `mod.rs`
- Screens: `menu.rs`, `setup.rs`, `session.rs` (animated), `results.rs`, `history.rs`
- Custom widgets: `breathing_bar.rs` (expands/contracts), `phase_map.rs`
- All screens are drawn fresh each tick; only changed areas update (ratatui handles diffing)

**Audio (src/audio.rs)**
- `Beeper` for phase-change beeps (uses terminal bell, no external deps)

### Why BreathingEngine is Copy

- Small struct: pattern ptr, 1 f64 (tempo), 5 u32/f64 fields, 1 bool
- Pass-by-copy more efficient than cloning
- Semantically matches a "value" type (like duration or counter)
- Makes defensive copies safe; no mutability surprises

### Animation in Session Screen

The breathing bar in `src/ui/widgets/breathing_bar.rs`:
- **Rising phases** (Inhale): Bar expands left→right
- **Falling phases** (Exhale): Bar contracts right→left
- **Steady phases** (Hold): Bar holds steady
- Uses Unicode blocks (█ filled, ░ hollow) for smooth effect
- Progress driven by `BreathingEngine::phase_progress()` (0.0 to 1.0)
- Color by phase: Cyan (inhale), Yellow (hold), Green (exhale)

### Session Tracking

Sessions are saved as JSON with structure matching `src/storage/schema.rs`:
- Session ID (UUID), timestamps, pattern name, duration, tempo
- Full event log (phase transitions, pause/resume)
- Calculated metrics (cycles, pauses, completion %)
- Index file at `~/.local/share/tui_breath/index.json` for fast history loading

## Key Design Constraints

1. **Terminal Minimum Size:** 60×24 characters. Check in UI layers before rendering.
2. **Tempo Range:** Clamped to 0.1–2.0 in `BreathingEngine::new()`
3. **No Panics in Production Code:** All I/O and user input paths return `Result<T>`
4. **Static Patterns:** Pattern definitions in `src/engine/patterns.rs` are immutable references (zero allocation cost)
5. **Pure Engine:** `BreathingEngine` has no side effects; all state updates are functional

## Testing

Unit tests are in `src/engine/breathing.rs` (tests module). They verify:
- Phase progression and timing
- Tempo effects on durations
- Completion detection
- Pause/resume behavior

Run with `cargo test`. Add `--nocapture` to see println! debug output.

## When Modifying...

**Adding a new breathing pattern:**
1. Define phases in `src/engine/patterns.rs` (add to `PATTERNS` array)
2. Menu automatically picks it up from the static list

**Changing UI layout:**
1. Modify the corresponding screen in `src/ui/*.rs`
2. All frames render in `impl Widget for ScreenWidget` (ratatui trait)
3. Test terminal size checks don't fail at 60×24

**Tweaking engine timing:**
1. Change phase durations in patterns or tempo multiplier in `src/engine/patterns.rs`
2. Tick interval is in `main.rs` (currently `Duration::from_millis(100)`)
3. Smaller ticks = smoother animation but more CPU; larger = jerky but lighter load

**Adding persistence:**
1. Update `src/storage/schema.rs` to add fields
2. Update save/load logic in `src/storage/store.rs`
3. Session data is human-readable JSON; easy to inspect `~/.local/share/tui_breath/sessions/`

## Dependencies Rationale

- `ratatui 0.27`: TUI framework; handles layout, widgets, rendering
- `crossterm 0.27`: Terminal I/O (Windows/Unix compatibility)
- `tokio 1.35`: Async runtime and timers
- `serde`/`serde_json`: Session serialization
- `chrono`: Timestamps
- `uuid`: Session IDs
- `dirs`: Cross-platform data directories (`~/.local/share` on Unix)
- `anyhow`: Error handling

## Performance Notes

- **Release binary:** ~8MB (with LTO enabled in `Cargo.toml`)
- **Memory:** <10MB during session
- **CPU:** <5% typical (single-threaded)
- **Render efficiency:** Only redraw on state change (ratatui optimizes)

## Common Pitfalls

1. **Forgetting to handle all `AppState` variants**: The enum forces exhaustive matching—use this to catch state transitions.
2. **Calling `.unwrap()` on I/O**: Always return `Result<T>` from file/storage operations.
3. **Modifying `BreathingEngine` in place during a session**: Engine is passed by copy; modifications are local only. Use `SessionManager` to persist state.
4. **Terminal size too small during testing**: Emulator must be ≥60×24. The app displays a resize prompt if too small.
5. **Assuming beeping works everywhere**: Terminal bell works locally and over SSH, but not all terminal emulators unmute it. This is intentional (respects system settings).

## General Coding Rules

### Test-Driven Development

TDD is the default paradigm. Write a failing test that pins the desired behaviour, then make it pass with the smallest change that works, then refactor with the test as the safety net.

Exceptions are rare and must be called out: throwaway spikes, pure config/docs changes, obvious one-line fixes. When unsure, write the test first.

Never invent a test framework or add a linter without confirming with the user first.

### Design: Functions vs. Structs

Use structs (OOP-style) when state and behaviour cluster together. Default to functions otherwise.

- A struct with one method and no fields is a function. Don't write it.
- If you'd pass the same bundle of state through three or more calls, it wants to be a struct.
- One-shot transforms, pure helpers, and small CLI utilities should stay as functions.

### Secrets & Private Data

**Never commit secrets, credentials, or private data.** Treat the repo as public-by-default.

- Do not stage `.env`, `.env.*` files.
- Do not commit credential or key files: `*.pem`, `*.key`, `*.p12`, `id_rsa`, or SSH private keys.
- Do not paste API keys, tokens, or passwords into source or config files. Reference them via environment variables and document the variable name only.
- If a secret has already been committed: rotate it first (it's compromised the moment it hit git history), then scrub history with `git filter-repo` / BFG, then force-push only after the rotation lands.

### Git Safety

- Do not force-push to `main` or `master`. Force-pushing a personal feature branch is fine.
- A user approving one push does **not** authorize subsequent pushes; ask again unless they've said so durably.
- Never skip hooks (`--no-verify`) unless the user explicitly requests it.

### JSON Formatting

All JSON files must use **2-space indentation** — never minified or single-line.
