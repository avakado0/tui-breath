use animate::Animate as _;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::audio::Beeper;
use crate::engine::SessionManager;
use crate::storage::Store;

#[derive(Debug, Clone)]
pub enum AppState {
    Menu(MenuState),
    Setup(SetupState),
    Session(SessionState),
    Results(ResultsState),
    History(HistoryState),
    Quitting,
}

#[derive(Debug, Clone)]
pub struct MenuState {
    pub selected_pattern_idx: usize,
}

#[derive(Debug, Clone)]
pub struct SetupState {
    pub pattern_idx: usize,
    pub duration_minutes: u32,
    pub tempo: f64,
    pub selected_field: SetupField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupField {
    Duration,
    Tempo,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub manager: SessionManager,
}

#[derive(Debug, Clone)]
pub struct ResultsState {
    pub manager: SessionManager,
    pub saved: bool,
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
}

impl App {
    pub fn new() -> Result<Self> {
        let storage = Store::new()?;
        Ok(Self {
            state: AppState::Menu(MenuState {
                selected_pattern_idx: 0,
            }),
            storage,
            beeper: Beeper::new(),
            previous_phase_idx: 0,
            session_animator: None,
        })
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        // Global quit
        if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
            || key.code == KeyCode::Char('q')
        {
            self.state = AppState::Quitting;
            return;
        }

        // Global beep toggle
        if key.code == KeyCode::Char('b') {
            self.beeper.toggle();
            return;
        }

        // State-specific handling
        match self.state.clone() {
            AppState::Menu(_) => self.handle_menu_key(key),
            AppState::Setup(_) => self.handle_setup_key(key),
            AppState::Session(_) => self.handle_session_key(key),
            AppState::Results(_) => self.handle_results_key(key),
            AppState::History(_) => self.handle_history_key(key),
            AppState::Quitting => {}
        }
    }

    fn handle_menu_key(&mut self, key: KeyEvent) {
        if let AppState::Menu(menu_state) = &self.state {
            let mut menu_state = menu_state.clone();
            use crate::engine::PATTERNS;

            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    menu_state.selected_pattern_idx = (menu_state.selected_pattern_idx + 1) % PATTERNS.len();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    menu_state.selected_pattern_idx = if menu_state.selected_pattern_idx == 0 {
                        PATTERNS.len() - 1
                    } else {
                        menu_state.selected_pattern_idx - 1
                    };
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.state = AppState::Setup(SetupState {
                        pattern_idx: menu_state.selected_pattern_idx,
                        duration_minutes: 5,
                        tempo: 1.0,
                        selected_field: SetupField::Duration,
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
                KeyCode::Char('+') => {
                    match setup_state.selected_field {
                        SetupField::Duration => setup_state.duration_minutes = (setup_state.duration_minutes + 1).min(60),
                        SetupField::Tempo => setup_state.tempo = (setup_state.tempo + 0.1).min(2.0),
                    }
                    self.state = AppState::Setup(setup_state);
                }
                KeyCode::Char('-') => {
                    match setup_state.selected_field {
                        SetupField::Duration => {
                            setup_state.duration_minutes = setup_state.duration_minutes.saturating_sub(1).max(1)
                        }
                        SetupField::Tempo => setup_state.tempo = (setup_state.tempo - 0.1).max(0.5),
                    }
                    self.state = AppState::Setup(setup_state);
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let pattern = &crate::engine::PATTERNS[setup_state.pattern_idx];
                    let duration_secs = (setup_state.duration_minutes as f64) * 60.0;
                    let manager = SessionManager::new(pattern, duration_secs, setup_state.tempo);
                    let first_phase = manager.engine.current_phase();
                    self.session_animator = Some(
                        crate::animator::SessionAnimator::for_phase(&first_phase.style, first_phase.name)
                    );
                    self.previous_phase_idx = manager.engine.current_phase_idx;
                    self.state = AppState::Session(SessionState { manager });
                }
                KeyCode::Esc => {
                    self.state = AppState::Menu(MenuState {
                        selected_pattern_idx: 0,
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
                    session.manager.toggle_pause();
                    self.state = AppState::Session(session);
                }
                KeyCode::Char('e') | KeyCode::Esc => {
                    let mut manager = session.manager;
                    manager.abandon();
                    self.session_animator = None;
                    self.state = AppState::Results(ResultsState { manager, saved: false });
                }
                _ => {}
            }
        }
    }

    fn handle_results_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s') => {
                if let AppState::Results(ref results) = self.state {
                    let _ = self.storage.save_session(&results.manager);
                }
                self.state = AppState::Menu(MenuState {
                    selected_pattern_idx: 0,
                });
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.state = AppState::Menu(MenuState {
                    selected_pattern_idx: 0,
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
                    });
                }
                _ => {}
            }
        }
    }

    pub fn on_tick(&mut self, delta_secs: f64) {
        if let AppState::Session(ref mut session_state) = self.state {
            let current_phase_idx = session_state.manager.engine.current_phase_idx;

            session_state.manager.engine.tick(delta_secs);

            let new_phase_idx = session_state.manager.engine.current_phase_idx;
            if current_phase_idx != new_phase_idx {
                self.beeper.beep();
                let phase = session_state.manager.engine.current_phase();
                let style = phase.style.clone();
                let name = phase.name;
                if let Some(anim) = self.session_animator.as_mut() {
                    let (r, g, b) = crate::animator::phase_color(&style);
                    anim.color_r.set(r);
                    anim.color_g.set(g);
                    anim.color_b.set(b);
                    anim.phase_label.set(name.to_string());
                    if matches!(style, crate::engine::patterns::PhaseStyle::Steady) {
                        anim.hold_pulse.set(0.65);
                    }
                }
            }

            if session_state.manager.engine.is_complete() {
                let mut manager = session_state.manager.clone();
                manager.complete();
                self.session_animator = None;
                self.state = AppState::Results(ResultsState { manager, saved: false });
            }
        }
    }

    pub fn is_quitting(&self) -> bool {
        matches!(self.state, AppState::Quitting)
    }
}
