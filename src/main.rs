use anyhow::Result;
use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io;
use std::time::Instant;
use tokio::time::Duration;

mod animator;
mod app;
mod audio;
mod engine;
mod events;
mod storage;
mod ui;

use app::App;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    setup_terminal()?;

    let result = run_app().await;

    restore_terminal()?;

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

async fn run_app() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let mut app = App::new()?;
    let tick_rate = Duration::from_millis(33);
    let mut interval = tokio::time::interval(tick_rate);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Non-blocking key event check
        if event::poll(Duration::from_millis(0))? {
            if let event::Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }

        // Tick interval
        interval.tick().await;
        let delta = last_tick.elapsed().as_secs_f64();
        last_tick = Instant::now();

        let engine_paused = matches!(&app.state, app::AppState::Session(s) if s.manager.engine.is_paused);
        if !engine_paused {
            animate::tick(33);
        }
        if let Some(anim) = app.session_animator.as_mut() {
            anim.animate();
        }

        app.on_tick(delta);

        if app.is_quitting() {
            break;
        }
    }

    Ok(())
}
