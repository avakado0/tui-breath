# Feature: Duration Units in Session Setup

## Context
The Session Setup screen currently hardcodes session duration as "X minutes" with `duration_minutes: u32` in `SetupState`. The user wants to switch to a unit-based duration input where 1 unit equals the duration of one complete breathing cycle (sum of all phase durations at tempo 1.0). This makes the session duration feel tied to the breathing pattern itself and allows intuitive cycle-level control.

## Approach
- Replace `duration_minutes` with `duration_units` in `SetupState`.
- Define 1 unit = sum of all phase durations for the selected pattern at base tempo.
- Bind both **Up/Down arrows** and **+/- keys** to increment/decrement the selected field. When `Duration` is selected, adjust units by ±1 (clamped 1–100). When `Tempo` is selected, adjust tempo by ±0.1 (existing behavior).
- In `setup.rs`, display the primary value as `"X units (≈ Y min)"` and show that estimated breathing cycles exactly equals the unit count.
- Default `duration_units` to 30, which gives ~5 minutes for the Diaphragmatic pattern and reasonable session lengths for others.

## Files to create / modify

### `src/app.rs` — modify
- **`SetupState`**: rename field `duration_minutes: u32` → `duration_units: u32`.
- **`handle_menu_key`**: set `duration_units: 30` when entering `AppState::Setup`.
- **`handle_setup_key`**:
  - Add `KeyCode::Up` handling: increment selected field value (Duration +1 unit, Tempo +0.1).
  - Add `KeyCode::Down` handling: decrement selected field value (Duration −1 unit, Tempo −0.1).
  - Clamp Duration to `1..=100`.
  - On `Enter`: compute `session_secs = (duration_units as f64) × total_pattern_secs / tempo`, then call `SessionManager::new(pattern, session_secs, tempo)`.

### `src/ui/setup.rs` — modify
- Replace the `"{} minutes"` line with `"{} units (≈ {:.1} min)"` derived from `setup_state.duration_units`.
- Compute `session_secs = units × total_pattern_secs (at tempo 1.0) / tempo`.
- Set `est_cycles = setup_state.duration_units` (exact, since 1 unit = 1 cycle).
- Change subtext line from `"total session length"` to `"1 unit = 1 complete breathing cycle"`.
- Update footer from `"[+/-] Adjust"` to `"[↑/↓] or [+/-] Adjust"`.

## Reuse
- `SetupField` enum (`Duration` / `Tempo`) — already used for card selection.
- Existing `+/-` tempo adjustment logic — extend to support Up/Down for both fields.
- Card border & selection highlight rendering — unchanged structure.
- Pattern phase iteration (`pattern.phases.iter()`) — already used for `total_pattern_secs`.

## Build sequence (ordered)

1. **Modify `src/app.rs`**
   - Rename `duration_minutes` → `duration_units` in `SetupState`.
   - Change default from `5` to `30` in `handle_menu_key`.
   - Add `KeyCode::Up` and `KeyCode::Down` arms in `handle_setup_key` (reuse the same ± logic as `+`/`-`).
   - Update `Enter` branch to compute `session_secs` from units instead of minutes.

2. **Modify `src/ui/setup.rs`**
   - Update the Duration card value line: compute `session_secs` from `duration_units` and display `"X units (≈ Y min)"`.
   - Set `est_cycles = setup_state.duration_units`.
   - Update subtext and footer strings.

3. **`cargo build`** — verify clean compilation.

4. **`cargo test`** — verify all existing tests still pass (engine timing tests are not affected by this UI/state change).

## Verification

1. **Compilation**: `cargo build` succeeds with no errors or warnings.
2. **Tests**: `cargo test` passes all existing tests.
3. **Display**: For Diaphragmatic pattern at tempo 1.0 (10s total cycle), the Duration card shows `"30 units (≈ 5.0 min)"`.
4. **Up/Down adjustment**: Pressing `↑` increments units by 1 (e.g., 30 → 31); pressing `↓` decrements by 1 (e.g., 30 → 29). Clamped at `1` minimum and `100` maximum.
5. **+/- still works**: `+` and `-` behave identically to `↑`/`↓` for whichever field (`Duration` or `Tempo`) is selected.
6. **Cycle estimate**: The line below the value shows `"≈ 30 breathing cycles"` exactly matching the unit count.
7. **Session start**: Pressing `Enter` starts a session whose actual duration matches `units × cycle_duration / tempo`.
