# Context — tui_breath

Domain glossary and shared concepts for the TUI Breathing Guide and its web port `breath4.life`.

## Glossary

**Pattern** — a named breathing technique made of an ordered list of phases that repeat as cycles. Built-in: 4-7-8, Box, Diaphragmatic. See `src/engine/patterns.rs` (TUI) and `breath4life-web/src/patterns.ts` (web).

**Phase** — one segment of a pattern. Has a name, a duration in seconds, a style, and optionally a channel.

**Phase style** — visual/temporal shape of a phase. One of `Rising` (inhale-like, bar fills), `Falling` (exhale-like, bar empties), or `Steady` (hold; bar holds at the extreme set by the previous non-Steady phase).

**Channel** — the airway used for a breathing phase: `nose` or `mouth`. Holds typically have no channel. Differs per pattern: 4-7-8 mouth-exhales (the "whoosh"), Box stays nose-only. Web UI surfaces this as a `[N]` / `[M]` glyph beside the phase label. Currently web-only; TUI may adopt later.

**Cycle** — one full pass through all phases of a pattern. The engine increments `cycle_count` each time the phase index wraps from last to first.

**Tempo** — multiplier on phase durations, clamped to `[0.1, 2.0]`. Effective phase duration = `phase.duration_secs / tempo`. Tempo 2.0 halves durations; 0.5 doubles them.

**Session** — one continuous run of a pattern at a given tempo for a target duration. Ends when total elapsed reaches the duration target or the user stops it.

**Tick** — one step of the engine clock. The engine accepts a `delta_secs` (Rust) / `deltaSecs` (web) parameter; it does not read wall-clock time itself. Pure function over (state, delta).

## Cross-project rules

- **Engine purity** — `BreathingEngine` (both projects) has no I/O, no clock, no DOM. Time always arrives as a parameter.
- **Patterns sync** — manual mirror between `src/engine/patterns.rs` and `breath4life-web/src/patterns.ts`. Web carries `channel`; Rust does not yet.
- **Colors** — phase color constants are duplicated:
  - `COLOR_INHALE = (0, 255, 255)` (cyan, Rising)
  - `COLOR_HOLD = (255, 230, 0)` (yellow, Steady)
  - `COLOR_EXHALE = (0, 220, 100)` (green, Falling)

## Project map

- `tui_breath/` — Rust TUI binary, ratatui + crossterm. The original implementation.
- `breath4life-web/` (sibling dir, separate repo) — Astro + plain TS web port at `breath4.life`. Pixel-art / phosphor-CRT aesthetic. AdSense-funded.
