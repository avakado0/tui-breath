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
use tokio::sync::watch;

mod animator;
mod app;
mod audio;
mod engine;
mod storage;
mod ui;
mod update;

use app::App;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    setup_terminal()?;

    let result = run_app().await;

    restore_terminal()?;

    match result {
        Ok(should_update) => {
            if should_update {
                std::process::Command::new("cargo")
                    .args(["install", "tui_breath", "--force"])
                    .status()?;
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Application error: {}", e);
            std::process::exit(1);
        }
    }
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

async fn run_app() -> Result<bool> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let (update_tx, update_rx) = watch::channel::<Option<String>>(None);
    tokio::task::spawn(async move {
        if let Some(v) = crate::update::check_for_update().await {
            let _ = update_tx.send(Some(v));
        }
    });

    let mut app = App::new(update_rx)?;
    let tick_rate = Duration::from_millis(33);
    let mut interval = tokio::time::interval(tick_rate);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(Duration::from_millis(0))? {
            if let event::Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }

        interval.tick().await;
        let delta = last_tick.elapsed().as_secs_f64();
        last_tick = Instant::now();

        if !app.session_is_paused() {
            if let Some(anim) = app.session_animator.as_mut() {
                anim.tick(delta);
            }
        }

        app.on_tick(delta);

        if app.is_quitting() {
            break;
        }
    }

    Ok(app.run_update)
}
