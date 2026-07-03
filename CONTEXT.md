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

**AudioMode** — one of three states for audio feedback: `Off` (silent), `Beep` (terminal bell), or `Tone` (ambient sine wave). User cycles through modes with `b` key.

**Tone Mode** — ambient audio using a continuous sine glide from 110 Hz to 220 Hz, driven by breath phase progress. Tracks `fill_ratio()` live; sustains at 110 Hz during breath holds and pauses. Requires an audio output device.

**Trend view** — a graph mode of the History screen, toggled with `g`, showing two stacked line charts over the session list: breath-hold-seconds-per-session (best hold per session) and sessions-per-day (practice frequency). Not a separate `AppState`; a view-mode flag on History, since both views read the same `IndexEntry` list.

**Time frame** — the visible window for the Trend view: `7d`, `30d`, or `all`, cycled with `t`. 7d/30d bucket one point per calendar day; `all` buckets one point per session.

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
