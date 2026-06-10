use anyhow::Result;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::audio::Beeper;
use crate::engine::{BreathHoldRuntime, SessionManager};
use crate::storage::Store;

#[derive(Debug, Clone)]
pub enum AppState {
    Menu(MenuState),
    Setup(SetupState),
    Session(SessionState),
    BodyMovements(BodyMovementsState),
    Results(ResultsState),
    History(HistoryState),
    Quitting,
}

#[derive(Debug, Clone)]
pub struct MenuState {
    pub selected_pattern_idx: usize,
    pub workout_mode: bool,
}

#[derive(Debug, Clone)]
pub struct SetupState {
    pub pattern_idx: usize,
    pub duration_units: u32,
    pub tempo: f64,
    pub selected_field: SetupField,
    pub workout_mode: bool,
}

#[derive(Debug, Clone)]
pub struct BodyMovementsState {
    pub session_state: SessionState,
    pub current_movement: usize,
    pub current_step: usize,
    pub step_elapsed: f64,
    pub prev_phase_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupField {
    Duration,
    Tempo,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub manager: SessionManager,
    pub mode: SessionMode,
    pub workout_mode: bool,
}

#[derive(Debug, Clone)]
pub enum SessionMode {
    Breathing,
    Holding(BreathHoldRuntime),
    DeepInhaleHold(BreathHoldRuntime),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SessionTickResult {
    pub phase_changed: bool,
    pub completed: bool,
}

impl SessionState {
    pub fn new(manager: SessionManager, workout_mode: bool) -> Self {
        Self {
            manager,
            mode: SessionMode::Breathing,
            workout_mode,
        }
    }

    pub fn is_paused(&self) -> bool {
        matches!(&self.mode, SessionMode::Breathing) && self.manager.engine.is_paused
    }

    pub fn start_hold(&mut self, now: DateTime<Utc>) -> bool {
        if !matches!(self.mode, SessionMode::Breathing) || self.manager.engine.is_paused {
            return false;
        }

        self.manager.start_hold(now);
        self.mode = SessionMode::Holding(BreathHoldRuntime::new(now));
        true
    }

    pub fn finish_hold(&mut self, now: DateTime<Utc>) -> bool {
        let mode = std::mem::replace(&mut self.mode, SessionMode::Breathing);
        if let SessionMode::Holding(runtime) = mode {
            self.manager.finish_hold(runtime.finish(now));
            true
        } else {
            self.mode = mode;
            false
        }
    }

    pub fn toggle_active_pause(&mut self) {
        if matches!(self.mode, SessionMode::Breathing) {
            self.manager.toggle_pause();
        }
    }

    pub fn tick(&mut self, delta_secs: f64) -> SessionTickResult {
        match &mut self.mode {
            SessionMode::Breathing => {
                let current_phase_idx = self.manager.engine.current_phase_idx;
                self.manager.engine.tick(delta_secs);

                SessionTickResult {
                    phase_changed: current_phase_idx != self.manager.engine.current_phase_idx,
                    completed: self.manager.engine.is_complete(),
                }
            }
            SessionMode::Holding(runtime) | SessionMode::DeepInhaleHold(runtime) => {
                runtime.tick(delta_secs);
                SessionTickResult::default()
            }
        }
    }

    pub fn finalize_active_hold(&mut self, now: DateTime<Utc>) {
        let mode = std::mem::replace(&mut self.mode, SessionMode::Breathing);
        match mode {
            SessionMode::Holding(runtime) => {
                self.manager.finish_hold(runtime.finish(now));
            }
            SessionMode::Breathing | SessionMode::DeepInhaleHold(_) => {}
        }
    }

    pub fn abandon(mut self, now: DateTime<Utc>) -> SessionManager {
        self.finalize_active_hold(now);
        self.manager.abandon();
        self.manager
    }

    pub fn complete(mut self, now: DateTime<Utc>) -> SessionManager {
        self.finalize_active_hold(now);
        self.manager.complete();
        self.manager
    }
}

#[derive(Debug, Clone)]
pub struct ResultsState {
    pub manager: SessionManager,
}

#[derive(Debug, Clone)]
pub struct HistoryState {
    pub sessions: Vec<crate::storage::schema::IndexEntry>,
    pub selected_idx: usize,
}

pub struct App {
    pub state: AppState,
    pub storage: Store,
    pub beeper: Beeper,
    previous_phase_idx: usize,
    pub session_animator: Option<crate::animator::SessionAnimator>,
    pub update_available: Option<String>,
    pub run_update: bool,
    pub update_rx: tokio::sync::watch::Receiver<Option<String>>,
}

impl App {
    pub fn new(update_rx: tokio::sync::watch::Receiver<Option<String>>) -> Result<Self> {
        let storage = Store::new()?;
        Ok(Self::new_with_store(storage, update_rx))
    }

    fn new_with_store(
        storage: Store,
        update_rx: tokio::sync::watch::Receiver<Option<String>>,
    ) -> Self {
        Self {
            state: AppState::Menu(MenuState {
                selected_pattern_idx: 0,
                workout_mode: true,
            }),
            storage,
            beeper: Beeper::new(),
            previous_phase_idx: 0,
            session_animator: None,
            update_available: None,
            run_update: false,
            update_rx,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            || key.code == KeyCode::Char('q')
        {
            self.state = AppState::Quitting;
            return;
        }

        if key.code == KeyCode::Char('b') {
            self.beeper.toggle();
            return;
        }

        match self.state.clone() {
            AppState::Menu(_) => self.handle_menu_key(key),
            AppState::Setup(_) => self.handle_setup_key(key),
            AppState::Session(_) => self.handle_session_key(key),
            AppState::BodyMovements(_) => self.handle_body_movements_key(key),
            AppState::Results(_) => self.handle_results_key(key),
            AppState::History(_) => self.handle_history_key(key),
            AppState::Quitting => {}
        }
    }

    fn handle_menu_key(&mut self, key: KeyEvent) {
        if self.update_available.is_some() {
            match key.code {
                KeyCode::Char('u') | KeyCode::Char('U') => {
                    self.run_update = true;
                    self.state = AppState::Quitting;
                }
                _ => {
                    self.update_available = None;
                }
            }
            return;
        }

        if let AppState::Menu(menu_state) = &self.state {
            let mut menu_state = menu_state.clone();
            use crate::engine::PATTERNS;

            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    menu_state.selected_pattern_idx =
                        (menu_state.selected_pattern_idx + 1) % PATTERNS.len();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    menu_state.selected_pattern_idx = if menu_state.selected_pattern_idx == 0 {
                        PATTERNS.len() - 1
                    } else {
                        menu_state.selected_pattern_idx - 1
                    };
                }
                KeyCode::Char('w') => {
                    menu_state.workout_mode = !menu_state.workout_mode;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.state = AppState::Setup(SetupState {
                        pattern_idx: menu_state.selected_pattern_idx,
                        duration_units: 30,
                        tempo: 1.0,
                        selected_field: SetupField::Duration,
                        workout_mode: menu_state.workout_mode,
                    });
                    return;
                }
                KeyCode::Char('h') => {
                    let sessions = self.storage.load_index().unwrap_or_default();
                    self.state = AppState::History(HistoryState {
                        sessions,
                        selected_idx: 0,
                    });
                    return;
                }
                _ => {}
            }
            self.state = AppState::Menu(menu_state);
        }
    }

    fn handle_setup_key(&mut self, key: KeyEvent) {
        if let AppState::Setup(setup_state) = &self.state {
            let mut setup_state = setup_state.clone();

            match key.code {
                KeyCode::Tab => {
                    setup_state.selected_field = match setup_state.selected_field {
                        SetupField::Duration => SetupField::Tempo,
                        SetupField::Tempo => SetupField::Duration,
                    };
                    self.state = AppState::Setup(setup_state);
                }
                KeyCode::Char('+') | KeyCode::Up => {
                    match setup_state.selected_field {
                        SetupField::Duration => {
                            setup_state.duration_units = (setup_state.duration_units + 1).min(100)
                        }
                        SetupField::Tempo => setup_state.tempo = (setup_state.tempo + 0.1).min(2.0),
                    }
                    self.state = AppState::Setup(setup_state);
                }
                KeyCode::Char('-') | KeyCode::Down => {
                    match setup_state.selected_field {
                        SetupField::Duration => {
                            setup_state.duration_units =
                                setup_state.duration_units.saturating_sub(1).max(1)
                        }
                        SetupField::Tempo => setup_state.tempo = (setup_state.tempo - 0.1).max(0.5),
                    }
                    self.state = AppState::Setup(setup_state);
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let pattern = &crate::engine::PATTERNS[setup_state.pattern_idx];
                    let base_cycle_secs: f64 = pattern.phases.iter().map(|p| p.duration_secs).sum();
                    let duration_secs =
                        setup_state.duration_units as f64 * base_cycle_secs / setup_state.tempo;
                    let manager = SessionManager::new(pattern, duration_secs, setup_state.tempo);
                    let first_phase = manager.engine.current_phase();
                    self.session_animator = Some(crate::animator::SessionAnimator::for_phase(
                        &first_phase.style,
                        first_phase.name,
                    ));
                    self.previous_phase_idx = manager.engine.current_phase_idx;
                    self.state = AppState::Session(SessionState::new(manager, setup_state.workout_mode));
                }
                KeyCode::Esc => {
                    self.state = AppState::Menu(MenuState {
                        selected_pattern_idx: 0,
                        workout_mode: setup_state.workout_mode,
                    });
                }
                _ => {}
            }
        }
    }

    fn handle_session_key(&mut self, key: KeyEvent) {
        if let AppState::Session(session_state) = &self.state {
            let mut session = session_state.clone();
            match key.code {
                KeyCode::Char('p') | KeyCode::Char(' ') => {
                    session.toggle_active_pause();
                    self.state = AppState::Session(session);
                }
                KeyCode::Char('h') => {
                    let changed = match session.mode {
                        SessionMode::Breathing => {
                            let started = session.start_hold(Utc::now());
                            if started {
                                self.set_hold_animator();
                            }
                            started
                        }
                        SessionMode::Holding(_) => {
                            let stopped = session.finish_hold(Utc::now());
                            if stopped {
                                let phase = session.manager.engine.current_phase();
                                self.set_phase_animator(&phase.style, phase.name);
                            }
                            stopped
                        }
                        SessionMode::DeepInhaleHold(_) => {
                            session.mode = SessionMode::Breathing;
                            let phase = session.manager.engine.current_phase();
                            self.set_phase_animator(&phase.style, phase.name);
                            true
                        }
                    };
                    if changed {
                        self.state = AppState::Session(session);
                    }
                }
                KeyCode::Char('d') => {
                    match session.mode {
                        SessionMode::Holding(_) => {
                            session.finish_hold(Utc::now());
                            session.mode =
                                SessionMode::DeepInhaleHold(BreathHoldRuntime::new(Utc::now()));
                            self.set_deep_inhale_animator();
                            self.state = AppState::Session(session);
                        }
                        SessionMode::DeepInhaleHold(_) => {
                            session.mode = SessionMode::Breathing;
                            let phase = session.manager.engine.current_phase();
                            self.set_phase_animator(&phase.style, phase.name);
                            self.state = AppState::Session(session);
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('m') if session.workout_mode => {
                    let phase_idx = session.manager.engine.current_phase_idx;
                    self.state = AppState::BodyMovements(BodyMovementsState {
                        current_step: phase_idx % 3,
                        prev_phase_idx: phase_idx,
                        session_state: session,
                        current_movement: 0,
                        step_elapsed: 0.0,
                    });
                }
                KeyCode::Char('e') | KeyCode::Esc => {
                    let manager = session.abandon(Utc::now());
                    self.session_animator = None;
                    self.state = AppState::Results(ResultsState { manager });
                }
                _ => {}
            }
        }
    }

    fn handle_body_movements_key(&mut self, key: KeyEvent) {
        if let AppState::BodyMovements(bm) = &self.state {
            let mut bm = bm.clone();
            match key.code {
                KeyCode::Char('1') => {
                    bm.current_movement = 0;
                    bm.current_step = 0;
                    bm.step_elapsed = 0.0;
                    self.state = AppState::BodyMovements(bm);
                }
                KeyCode::Char('2') => {
                    bm.current_movement = 1;
                    bm.current_step = 0;
                    bm.step_elapsed = 0.0;
                    self.state = AppState::BodyMovements(bm);
                }
                KeyCode::Char('3') => {
                    bm.current_movement = 2;
                    bm.current_step = 0;
                    bm.step_elapsed = 0.0;
                    self.state = AppState::BodyMovements(bm);
                }
                KeyCode::Char('m') | KeyCode::Esc => {
                    self.state = AppState::Session(bm.session_state);
                }
                _ => {}
            }
        }
    }

    fn handle_results_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('f') => {
                if let AppState::Results(ref results) = self.state {
                    let _ = self
                        .storage
                        .forget_session(&results.manager.session_id.to_string());
                }
                self.state = AppState::Menu(MenuState {
                    selected_pattern_idx: 0,
                    workout_mode: true,
                });
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.state = AppState::Menu(MenuState {
                    selected_pattern_idx: 0,
                    workout_mode: true,
                });
            }
            _ => {}
        }
    }

    fn handle_history_key(&mut self, key: KeyEvent) {
        if let AppState::History(history_state) = &self.state {
            let mut history = history_state.clone();
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if history.selected_idx < history.sessions.len().saturating_sub(1) {
                        history.selected_idx += 1;
                    }
                    self.state = AppState::History(history);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if history.selected_idx > 0 {
                        history.selected_idx -= 1;
                    }
                    self.state = AppState::History(history);
                }
                KeyCode::Esc => {
                    self.state = AppState::Menu(MenuState {
                        selected_pattern_idx: 0,
                        workout_mode: true,
                    });
                }
                _ => {}
            }
        }
    }

    pub fn on_tick(&mut self, delta_secs: f64) {
        if self.update_available.is_none() {
            if let Some(v) = self.update_rx.borrow().clone() {
                self.update_available = Some(v);
            }
        }

        let mut phase_update = None;
        let mut completed_manager = None;

        if let AppState::Session(ref mut session_state) = self.state {
            let tick_result = session_state.tick(delta_secs);

            if tick_result.phase_changed {
                let phase = session_state.manager.engine.current_phase();
                phase_update = Some((
                    phase.style,
                    phase.name,
                    session_state.manager.engine.current_phase_idx,
                ));
            }

            if tick_result.completed {
                completed_manager = Some(session_state.clone().complete(Utc::now()));
            }
        }

        if let AppState::BodyMovements(ref mut bm) = self.state {
            let tick_result = bm.session_state.tick(delta_secs);
            let current_phase_idx = bm.session_state.manager.engine.current_phase_idx;
            if tick_result.phase_changed {
                let phase = bm.session_state.manager.engine.current_phase();
                phase_update = Some((
                    phase.style,
                    phase.name,
                    current_phase_idx,
                ));
                // Advance step in sync with breath phase
                bm.current_step = current_phase_idx % 3;
                bm.step_elapsed = 0.0;
                bm.prev_phase_idx = current_phase_idx;
            }
            bm.step_elapsed += delta_secs;
        }

        if let Some((style, label, phase_idx)) = phase_update {
            self.beeper.beep();
            self.set_phase_animator(&style, label);
            self.previous_phase_idx = phase_idx;
        }

        if let Some(manager) = completed_manager {
            let _ = self.storage.save_session(&manager);
            self.session_animator = None;
            self.state = AppState::Results(ResultsState { manager });
        }
    }

    pub fn session_is_paused(&self) -> bool {
        matches!(&self.state, AppState::Session(session) if session.is_paused())
    }

    pub fn is_quitting(&self) -> bool {
        matches!(self.state, AppState::Quitting)
    }

    fn set_phase_animator(&mut self, style: &crate::engine::patterns::PhaseStyle, label: &str) {
        if let Some(anim) = self.session_animator.as_mut() {
            let (r, g, b) = crate::animator::phase_color(style);
            anim.color_r.set(r);
            anim.color_g.set(g);
            anim.color_b.set(b);
            anim.phase_label.set(label.to_string());
            if matches!(style, crate::engine::patterns::PhaseStyle::Steady) {
                anim.hold_pulse.set(0.65);
            }
        } else {
            self.session_animator = Some(crate::animator::SessionAnimator::for_phase(style, label));
        }
    }

    fn set_hold_animator(&mut self) {
        const HOLD_R: f64 = 255.0;
        const HOLD_G: f64 = 120.0;
        const HOLD_B: f64 = 140.0;

        if let Some(anim) = self.session_animator.as_mut() {
            anim.color_r.set(HOLD_R);
            anim.color_g.set(HOLD_G);
            anim.color_b.set(HOLD_B);
            anim.phase_label.set("Breath Hold".to_string());
            anim.hold_pulse.set(0.55);
        } else {
            let mut anim = crate::animator::SessionAnimator::for_phase(
                &crate::engine::patterns::PhaseStyle::Steady,
                "Breath Hold",
            );
            anim.color_r.set(HOLD_R);
            anim.color_g.set(HOLD_G);
            anim.color_b.set(HOLD_B);
            anim.hold_pulse.set(0.55);
            self.session_animator = Some(anim);
        }
    }

    fn set_deep_inhale_animator(&mut self) {
        const DEEP_R: f64 = 255.0;
        const DEEP_G: f64 = 160.0;
        const DEEP_B: f64 = 40.0;

        if let Some(anim) = self.session_animator.as_mut() {
            anim.color_r.set(DEEP_R);
            anim.color_g.set(DEEP_G);
            anim.color_b.set(DEEP_B);
            anim.phase_label.set("Deep Inhale Hold".to_string());
            anim.hold_pulse.set(0.85);
        } else {
            let mut anim = crate::animator::SessionAnimator::for_phase(
                &crate::engine::patterns::PhaseStyle::Steady,
                "Deep Inhale Hold",
            );
            anim.color_r.set(DEEP_R);
            anim.color_g.set(DEEP_G);
            anim.color_b.set(DEEP_B);
            anim.hold_pulse.set(0.85);
            self.session_animator = Some(anim);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animator::SessionAnimator;
    use crate::audio::Beeper;
    use crate::engine::PATTERNS;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use uuid::Uuid;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn test_app() -> App {
        let temp_dir = std::env::temp_dir().join(format!("tui_breath_test_{}", Uuid::new_v4()));
        let store = Store::new_in(temp_dir).unwrap();
        let manager = SessionManager::new(&PATTERNS[1], 60.0, 1.0);
        let first_phase = manager.engine.current_phase();
        let (_tx, rx) = tokio::sync::watch::channel(None);

        App {
            state: AppState::Session(SessionState::new(manager, false)),
            storage: store,
            beeper: Beeper::new(),
            previous_phase_idx: 0,
            session_animator: Some(SessionAnimator::for_phase(
                &first_phase.style,
                first_phase.name,
            )),
            update_available: None,
            run_update: false,
            update_rx: rx,
        }
    }

    #[test]
    fn results_forget_removes_saved_session() {
        let temp_dir = std::env::temp_dir().join(format!("tui_breath_test_{}", Uuid::new_v4()));
        let store = Store::new_in(temp_dir).unwrap();
        let mut manager = SessionManager::new(&PATTERNS[0], 60.0, 1.0);
        manager.complete();
        let session_id = manager.session_id.to_string();
        store.save_session(&manager).unwrap();
        let session_path = store.sessions_dir().join(format!("{session_id}.json"));
        assert!(session_path.exists());

        let (_tx, rx) = tokio::sync::watch::channel(None);
        let mut app = App {
            state: AppState::Results(ResultsState { manager }),
            storage: store,
            beeper: Beeper::new(),
            previous_phase_idx: 0,
            session_animator: None,
            update_available: None,
            run_update: false,
            update_rx: rx,
        };

        app.on_key(key(KeyCode::Char('f')));

        assert!(!session_path.exists());
        assert!(app.storage.load_index().unwrap().is_empty());
        match app.state {
            AppState::Menu(menu) => assert!(menu.workout_mode),
            _ => panic!("expected menu state"),
        }
    }

    #[test]
    fn startup_defaults_to_workout_mode() {
        let temp_dir = std::env::temp_dir().join(format!("tui_breath_test_{}", Uuid::new_v4()));
        let store = Store::new_in(temp_dir).unwrap();
        let (_tx, rx) = tokio::sync::watch::channel(None);

        let app = App::new_with_store(store, rx);

        match app.state {
            AppState::Menu(menu) => assert!(menu.workout_mode),
            _ => panic!("expected menu state"),
        }
    }

    #[test]
    fn breath_hold_freezes_breathing_and_resumes_same_progress() {
        let mut app = test_app();
        app.on_tick(1.0);

        let (phase_idx, phase_elapsed, total_elapsed) = match &app.state {
            AppState::Session(session) => (
                session.manager.engine.current_phase_idx,
                session.manager.engine.phase_elapsed_secs,
                session.manager.engine.total_elapsed_secs,
            ),
            _ => panic!("expected session state"),
        };

        app.on_key(key(KeyCode::Char('h')));
        app.on_tick(2.0);

        match &app.state {
            AppState::Session(session) => {
                assert!(matches!(session.mode, SessionMode::Holding(_)));
                assert_eq!(session.manager.engine.current_phase_idx, phase_idx);
                assert!((session.manager.engine.phase_elapsed_secs - phase_elapsed).abs() < 0.001);
                assert!((session.manager.engine.total_elapsed_secs - total_elapsed).abs() < 0.001);
            }
            _ => panic!("expected session state"),
        }

        app.on_key(key(KeyCode::Char('h')));
        app.on_tick(1.0);

        match &app.state {
            AppState::Session(session) => {
                assert!(matches!(session.mode, SessionMode::Breathing));
                assert!(session.manager.engine.total_elapsed_secs > total_elapsed);
            }
            _ => panic!("expected session state"),
        }
    }

    #[test]
    fn pause_key_is_ignored_during_hold_mode() {
        let mut app = test_app();
        app.on_key(key(KeyCode::Char('h')));
        app.on_key(key(KeyCode::Char('p')));
        app.on_tick(1.5);

        match &app.state {
            AppState::Session(session) => {
                assert_eq!(session.manager.engine.pause_count, 0);
                assert!(
                    matches!(&session.mode, SessionMode::Holding(runtime) if runtime.elapsed_secs >= 1.5)
                );
            }
            _ => panic!("expected session state"),
        }
    }

    #[test]
    fn deep_inhale_starts_from_hold_and_returns_to_breathing() {
        let mut app = test_app();
        app.on_key(key(KeyCode::Char('h')));
        app.on_tick(1.0);
        app.on_key(key(KeyCode::Char('d')));
        app.on_tick(2.0);

        match &app.state {
            AppState::Session(session) => {
                assert!(matches!(session.mode, SessionMode::DeepInhaleHold(_)));
                assert!(
                    matches!(&session.mode, SessionMode::DeepInhaleHold(rt) if rt.elapsed_secs >= 2.0)
                );
            }
            _ => panic!("expected session state"),
        }

        app.on_key(key(KeyCode::Char('d')));

        match &app.state {
            AppState::Session(session) => {
                assert!(matches!(session.mode, SessionMode::Breathing));
            }
            _ => panic!("expected session state"),
        }
    }

    #[test]
    fn deep_inhale_hold_not_recorded_as_attempt() {
        let mut app = test_app();
        app.on_key(key(KeyCode::Char('h')));
        app.on_tick(1.0);
        app.on_key(key(KeyCode::Char('d')));
        app.on_tick(2.0);
        app.on_key(key(KeyCode::Char('d')));
        app.on_key(key(KeyCode::Char('e')));

        match &app.state {
            AppState::Results(results) => {
                assert_eq!(results.manager.hold_attempt_count(), 1);
            }
            _ => panic!("expected results state"),
        }
    }

    #[test]
    fn ending_session_from_hold_captures_attempt() {
        let mut app = test_app();
        app.on_key(key(KeyCode::Char('h')));
        app.on_tick(1.4);
        app.on_key(key(KeyCode::Char('e')));

        match &app.state {
            AppState::Results(results) => {
                assert_eq!(results.manager.hold_attempt_count(), 1);
                assert_eq!(
                    results.manager.session_status(),
                    Some(crate::engine::session::SessionOutcome::Abandoned)
                );
                assert!(results.manager.best_hold_seconds().unwrap() >= 1.4);
            }
            _ => panic!("expected results state"),
        }
    }
}
