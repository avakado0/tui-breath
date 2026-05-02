use ratatui::prelude::*;

#[derive(Debug, Clone)]
pub struct PhaseMap {
    pub progress: f64,
}

impl PhaseMap {
    pub fn new(progress: f64) -> Self {
        Self { progress }
    }
}

impl Widget for PhaseMap {
    fn render(self, _area: Rect, _buf: &mut Buffer) {
        // Stub
    }
}
