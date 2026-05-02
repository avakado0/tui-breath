# TUI Breathing Guide — Architecture Diagrams

This directory contains Mermaid diagrams that visualize different aspects of the tui_breath application architecture, data flow, and state management.

## Diagrams Overview

### 1. Architecture Diagram (`architecture.svg`)

**What it shows:** The complete module structure and layer separation of the application.

**Layers:**
- **Main Loop Layer** (blue): Terminal setup, event loop, async runtime
- **Application Layer** (red): App state machine and audio control
- **Breathing Engine** (green): Pure computation core (no I/O)
- **User Interface Layer** (light red): Screen renderers and custom widgets
- **Storage Layer** (orange): File I/O and JSON serialization
- **External Dependencies** (purple): Ratatui, Crossterm, Tokio, Serde

**Key insights:**
- The breathing engine is isolated and pure (no side effects)
- UI layer dispatches to different screens via a router
- Storage is entirely separate from the engine
- All external dependencies are grouped at the bottom

**Use this diagram when:**
- Explaining the project structure to new contributors
- Planning refactoring that affects module organization
- Understanding how layers communicate

---

### 2. State Machine Diagram (`state_machine.svg`)

**What it shows:** Valid `AppState` enum transitions and their triggers.

**States:**
- `Menu` — Pattern selection screen
- `Setup` — Duration and tempo configuration
- `Session` — Active breathing with animation
- `Results` — Post-session metrics display
- `History` — Session history browser
- `Quitting` — Exit state

**Transitions include:**
- Keyboard input triggers (Enter, Esc, q, h, j, k, etc.)
- Conditions (e.g., "session complete")
- State-specific behavior (pause/resume during session)

**Key insights:**
- All states are exhaustively matched (enforced by Rust's type system)
- Quitting always leads to exit
- Session state has sub-behaviors (pause, beep toggle, early exit)
- Menu can transition to both Setup and History

**Use this diagram when:**
- Validating state transition logic
- Testing all possible user flows
- Documenting valid paths through the app
- Debugging unexpected state transitions

---

### 3. Data Flow Diagram (`data_flow.svg`)

**What it shows:** How session data flows through the system during an active breathing session.

**Flow:**
1. Session starts with pattern, duration, tempo
2. `Event::Tick` fires every 100ms
3. `BreathingEngine.tick()` advances timers (Copy struct, immutable)
4. Calculation step computes progress, cycles, phase transitions
5. Event logging records phase changes and user actions
6. UI rendering reads engine state to draw animations
7. User interactions (pause, beep, end) update engine state
8. On completion, metrics are calculated and session is saved

**Key insights:**
- `BreathingEngine` is a Copy struct passed by value, not mutable reference
- Events are logged incrementally (low-latency append)
- All I/O (file writes) happens on completion, not during session
- 100ms tick rate drives the entire pipeline

**Use this diagram when:**
- Understanding session lifecycle
- Optimizing timing or performance
- Debugging pause/resume behavior
- Adding new session features (e.g., custom metrics)

---

### 4. Module Dependency Diagram (`module_dependency.svg`)

**What it shows:** Which modules depend on which, revealing coupling strength.

**Module Groups:**
- **Core**: Main entry, app state machine, event types, audio
- **Engine**: Pure computation (breathing, patterns, session manager)
- **UI**: Screen renderers and custom widgets
- **Storage**: File I/O and JSON serialization
- **External**: Ratatui, Crossterm, Tokio, Serde

**Coupling patterns:**
- App (red) is the hub — depends on engine, UI, storage, audio
- Engine (green) is self-contained — only depends on patterns
- UI depends heavily on Ratatui (rendering)
- Storage depends on Serde (serialization)

**Key insights:**
- No circular dependencies (acyclic)
- Engine has no external dependencies (zero-cost)
- UI and Storage are loosely coupled (only via App)
- Main loop is minimal and hands off to App

**Use this diagram when:**
- Identifying tight coupling to refactor
- Planning dependency injection
- Optimizing compile times
- Adding new modules

---

### 5. Event Flow Diagram (`event_flow.svg`)

**What it shows:** How events propagate from input sources through handlers to output.

**Flow:**
1. **Input Sources**: Tick (100ms), Keyboard, Resize
2. **Event Generation**: `poll_events()` async, queued in Event enum
3. **Handling**: `handle_event()` matches state and dispatches
4. **State Update**: Engine ticks or state transitions
5. **Rendering**: UI draws based on new state
6. **Output**: Terminal display refreshes

**Key insights:**
- Single event loop (main.rs) for all input sources
- State machine ensures valid transitions
- Session state routes to engine ticks
- Rendering happens after every event
- User sees output, which generates next input (feedback loop)

**Use this diagram when:**
- Debugging input handling issues
- Optimizing event loop performance
- Understanding event ordering
- Adding new keyboard shortcuts

---

## Color Legend

| Color | Meaning |
|-------|---------|
| 🔵 Blue | Main loop, entry point, event sources |
| 🟢 Green | Breathing engine (pure computation) |
| 🔴 Red | App state machine, handlers |
| 🟠 Orange | Storage and persistence |
| 🟣 Purple | External dependencies |
| 🔴 Light Red | UI and rendering |

---

## How the Diagrams Relate

```
Architecture (modules)
    ↓ (defines)
State Machine (AppState enum)
    ↓ (driven by)
Event Flow (keyboard + ticks)
    ↓ (updates)
Data Flow (session state)
    ↓ (shows)
Module Dependency (coupling)
```

The **Architecture** shows the "what" (which modules exist).
The **State Machine** shows the "when" (valid transitions).
The **Event Flow** shows the "how" (events trigger changes).
The **Data Flow** shows the "why" (session lifecycle).
The **Module Dependency** shows the "risks" (tight coupling).

---

## Rendering & Updating

### Regenerate all diagrams:
```bash
mmdc -i architecture.mmd -o architecture.svg
mmdc -i state_machine.mmd -o state_machine.svg
mmdc -i data_flow.mmd -o data_flow.svg
mmdc -i module_dependency.mmd -o module_dependency.svg
mmdc -i event_flow.mmd -o event_flow.svg
```

### View in browser:
```bash
open architecture.svg  # macOS
xdg-open architecture.svg  # Linux
start architecture.svg  # Windows
```

### Edit diagrams:
- Edit the `.mmd` files with any text editor
- Re-render with mmdc to update `.svg` files
- Commit both `.mmd` (source) and `.svg` (rendered) to git

---

## Integration Points

### Architecture Diagram → CLAUDE.md
The architecture diagram should match the "Module Structure" section in CLAUDE.md. Update both when major refactoring occurs.

### State Machine Diagram → Testing
Each state transition should have at least one test. Use the state machine diagram to ensure coverage:
- Menu → Setup (select pattern)
- Setup → Session (start breathing)
- Session → Results (complete or early exit)
- Results → Menu (save or skip)
- History → Menu (back)

### Data Flow Diagram → Performance
The data flow diagram identifies bottlenecks:
- 100ms tick rate is the limiting factor
- Event logging should be O(1) append
- File I/O happens only on completion (not during session)
- Rendering is diff-based (ratatui optimizes)

### Module Dependency Diagram → Refactoring
Before refactoring, check if the dependency diagram changes:
- Adding dependencies to Engine? (Bad — it's supposed to be pure)
- Circular dependency? (Very bad — restructure)
- UI depends on Storage? (Bad — go through App)
- All dependencies point to App or Engine? (Good — clear hub-and-spoke)

---

## Version History

| Date | Version | Changes |
|------|---------|---------|
| Apr 14, 2026 | 1.0 | Initial diagrams generated for tui_breath |

---

## For Developers

When making changes that affect these diagrams, update them accordingly:

- **Added a new module?** → Update Architecture diagram
- **Changed state transitions?** → Update State Machine diagram
- **New event type?** → Update Event Flow diagram
- **Changed session lifecycle?** → Update Data Flow diagram
- **Added/removed dependencies?** → Update Module Dependency diagram

Keep diagrams and code synchronized for accurate documentation.
