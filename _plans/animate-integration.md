# Plan: Animate Library Integration — Enhanced Breathing Animations

## Context

The current TUI breathing app uses fully manual, linear animation: a bar fills left→right or drains right→left using raw `engine.phase_progress()` (0.0–1.0), and phase color changes are instant. The `animate` library was cloned at `/home/draco/docs/tui_breath/animate/` to add easing curves, smooth color crossfades, a hold-phase pulse, and a typewriter phase label — replacing the previous simple animation system with a richer visual experience.

The plan modifies existing source files in place (new version replaces old inline session render logic).

---

## Architecture

- **`BreathingEngine` stays as timing authority** — phase transitions, phase_progress, completion
- **`animate` adds 4 visual effects on top:**
  1. **Bar easing** — `cubic_in_out(engine.phase_progress())` applied in render (no animate state needed)
  2. **Color crossfade** — 3 animated `f64` fields (R/G/B) fade between Cyan/Yellow/Green over 300ms on phase change
  3. **Hold pulse** — `#[alternate]` brightness multiplier (0.85↔1.0, 1200ms) applied to bar color during Steady phases
  4. **Typewriter label** — `#[once]` `String` field reveals phase name character-by-character over 300ms on phase change
- **`animate::tick(100)`** called in main loop each tick, skipped when engine is paused

**Note:** Do NOT use `features = ["ratatui"]` on the animate dependency — the app uses ratatui 0.27 but animate-core requires ratatui 0.30. We animate `f64` R/G/B channels and construct `Color::Rgb(r as u8, g as u8, b as u8)` in render.

**Note:** Easing functions are re-exported at `animate::cubic_in_out` directly (not `animate::easing::...`).

---

## Critical Files

- `Cargo.toml` — add animate dependency
- `src/animator.rs` — **new file**: `SessionAnimator` struct with `#[animate]` macro
- `src/app.rs` — add `session_animator: Option<SessionAnimator>` field, trigger phase animations
- `src/main.rs` — add `mod animator`, `animate::tick()`, `anim.animate()` calls
- `src/ui/session.rs` — consume animated values in render

---

## Step-by-Step Implementation

### 1. Cargo.toml

Add to `[dependencies]`:
```toml
animate = { path = "animate/animate" }
```
No feature flags. Verify with `cargo check`.

---

### 2. Create `src/animator.rs`

```rust
use animate::animate;
use crate::engine::patterns::PhaseStyle;

const COLOR_INHALE: (f64, f64, f64) = (0.0, 255.0, 255.0);   // Cyan
const COLOR_HOLD:   (f64, f64, f64) = (255.0, 230.0, 0.0);   // Yellow
const COLOR_EXHALE: (f64, f64, f64) = (0.0, 220.0, 100.0);   // Green

pub fn phase_color(style: &PhaseStyle) -> (f64, f64, f64) {
    match style {
        PhaseStyle::Rising  => COLOR_INHALE,
        PhaseStyle::Steady  => COLOR_HOLD,
        PhaseStyle::Falling => COLOR_EXHALE,
    }
}

#[animate]
pub struct SessionAnimator {
    #[once(duration = 300, easing = quad_in_out)]
    pub color_r: f64,

    #[once(duration = 300, easing = quad_in_out)]
    pub color_g: f64,

    #[once(duration = 300, easing = quad_in_out)]
    pub color_b: f64,

    #[once(duration = 300)]
    pub phase_label: String,

    #[alternate(duration = 1200, easing = quad_in_out)]
    pub hold_pulse: f64,
}

impl SessionAnimator {
    pub fn for_phase(style: &PhaseStyle, label: &str) -> Self {
        let (r, g, b) = phase_color(style);
        let mut anim = SessionAnimator::new(r, g, b, label.to_string(), 1.0);
        anim.hold_pulse.set(0.85);  // Start alternate oscillation
        anim
    }
}
```

---

### 3. `src/app.rs` Changes

**Add field to `App`:**
```rust
pub session_animator: Option<crate::animator::SessionAnimator>,
```
Initialize to `None` in `App::new()`.

**In `handle_setup_key()`, when starting session (Enter pressed):** after creating `manager`, before/after creating `AppState::Session`:
```rust
let first_phase = manager.engine.current_phase();
self.session_animator = Some(
    crate::animator::SessionAnimator::for_phase(&first_phase.style, first_phase.name)
);
self.state = AppState::Session(SessionState { manager });
```

**In `on_tick()`, after detecting phase change** (existing `phase_changed` detection):
```rust
if phase_changed {
    self.beeper.beep();
    // Trigger animate transitions
    let phase = session_state.manager.engine.current_phase();
    if let Some(anim) = self.session_animator.as_mut() {
        let (r, g, b) = crate::animator::phase_color(&phase.style);
        anim.color_r.set(r);
        anim.color_g.set(g);
        anim.color_b.set(b);
        anim.phase_label.set(phase.name.to_string());
        if matches!(phase.style, PhaseStyle::Steady) {
            anim.hold_pulse.set(0.85);
        }
    }
}
```

**On session end** (completion and `e`-key abandon): add `self.session_animator = None;`.

---

### 4. `src/main.rs` Changes

Add near top: `mod animator;`

In the event loop, each tick — before `app.on_tick(delta)`:
```rust
let engine_paused = matches!(&app.state, AppState::Session(s) if s.manager.engine.is_paused);
if !engine_paused {
    animate::tick(100);
}
if let Some(anim) = app.session_animator.as_mut() {
    anim.animate();
}
```

---

### 5. `src/ui/session.rs` Changes

Pass `session_animator` into the draw function or access via `app`:

**Replace instant color logic:**
```rust
// Old: Color::Cyan / Color::Yellow / Color::Green match

// New: read animated RGB channels
let (cr, cg, cb) = if let Some(anim) = &app.session_animator {
    (*anim.color_r as u8, *anim.color_g as u8, *anim.color_b as u8)
} else {
    match engine.current_phase().style {
        PhaseStyle::Rising  => (0, 255, 255),
        PhaseStyle::Steady  => (255, 230, 0),
        PhaseStyle::Falling => (0, 220, 100),
    }
};
// Apply hold pulse multiplier only during Steady
let (cr, cg, cb) = if matches!(engine.current_phase().style, PhaseStyle::Steady) {
    let pulse = app.session_animator.as_ref().map(|a| *a.hold_pulse).unwrap_or(1.0);
    ((cr as f64 * pulse) as u8, (cg as f64 * pulse) as u8, (cb as f64 * pulse) as u8)
} else {
    (cr, cg, cb)
};
let color = Color::Rgb(cr, cg, cb);
```

**Apply bar easing:**
```rust
// Old:
let progress = engine.phase_progress();

// New: (cubic_in_out re-exported at animate:: directly)
let progress = animate::cubic_in_out(engine.phase_progress());
```

**Replace phase label with typewriter:**
```rust
// Old:
let phase_label = current_phase.name;

// New:
let phase_label_owned;
let phase_label = if let Some(anim) = &app.session_animator {
    phase_label_owned = anim.phase_label.get().clone();
    phase_label_owned.as_str()
} else {
    current_phase.name
};
```

---

## Verification

```bash
cargo check                          # Verify it compiles cleanly
cargo build                          # Debug build
cargo run                            # Run and manually test
cargo test                           # Unit tests still pass
```

Manual test checklist:
1. Start a session → bar fills with eased curve (should feel organic, not linear)
2. Phase transitions → color crossfades smoothly over ~300ms (not instant)
3. Phase label appears with typewriter character-reveal effect
4. During Hold phase → subtle brightness pulse visible on the full bar
5. Pause session mid-color-transition → everything freezes cleanly
6. Resume → animations continue from freeze point
7. End session with `e` → Results screen renders correctly, no crash
8. Run to completion → Results screen, save session, history
