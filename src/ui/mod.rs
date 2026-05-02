use ratatui::prelude::*;

use crate::app::{App, AppState};

pub mod menu;
pub mod setup;
pub mod session;
pub mod results;
pub mod history;
pub mod widgets;

pub fn draw(f: &mut Frame, app: &App) {
    match &app.state {
        AppState::Menu(_) => menu::draw(f, app),
        AppState::Setup(_) => setup::draw(f, app),
        AppState::Session(_) => session::draw(f, app),
        AppState::Results(_) => results::draw(f, app),
        AppState::History(_) => history::draw(f, app),
        AppState::Quitting => {
            // Draw nothing on quit
        }
    }
}
