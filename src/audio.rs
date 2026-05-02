use std::sync::{Arc, Mutex};

/// Beeper for audio feedback between breathing phases
/// Uses terminal bell (0x07 character) for cross-platform compatibility
pub struct Beeper {
    enabled: Arc<Mutex<bool>>,
}

impl Beeper {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(Mutex::new(false)),
        }
    }

    pub fn toggle(&self) {
        if let Ok(mut enabled) = self.enabled.lock() {
            *enabled = !*enabled;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.lock().map(|e| *e).unwrap_or(false)
    }

    /// Trigger a beep if enabled
    pub fn beep(&self) {
        if !self.is_enabled() {
            return;
        }

        // Spawn beep in background thread to not block UI
        let enabled = Arc::clone(&self.enabled);
        std::thread::spawn(move || {
            if let Ok(true) = enabled.lock().map(|e| *e) {
                play_beep();
            }
        });
    }
}

/// Play a beep using the terminal bell character
/// This works on all platforms (terminal, over SSH, etc.)
fn play_beep() {
    use std::io::Write;

    // Send the bell character (BEL, ASCII 0x07)
    // Most terminals will produce an audible beep
    // Some might produce a visual bell instead
    print!("\x07");
    let _ = std::io::stdout().flush();
}

impl Clone for Beeper {
    fn clone(&self) -> Self {
        Self {
            enabled: Arc::clone(&self.enabled),
        }
    }
}
