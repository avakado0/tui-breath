use ratatui::prelude::*;

#[derive(Debug, Clone)]
pub struct BreathingBar {
    pub progress: f64,
}

impl BreathingBar {
    pub fn new(progress: f64) -> Self {
        Self { progress }
    }
}

impl Widget for BreathingBar {
    fn render(self, _area: Rect, _buf: &mut Buffer) {
        // Stub
    }
}
