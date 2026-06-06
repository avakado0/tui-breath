use ratatui::prelude::*;

use crate::app::{App, AppState};

pub mod body_movements;
pub mod history;
pub mod menu;
pub mod results;
pub mod session;
pub mod setup;

pub fn draw(f: &mut Frame, app: &App) {
    match &app.state {
        AppState::Menu(_) => menu::draw(f, app),
        AppState::Setup(_) => setup::draw(f, app),
        AppState::Session(_) => session::draw(f, app),
        AppState::BodyMovements(_) => body_movements::draw(f, app),
        AppState::Results(_) => results::draw(f, app),
        AppState::History(_) => history::draw(f, app),
        AppState::Quitting => {}
    }
}
