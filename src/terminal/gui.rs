use std::time::Instant;
use ratatui::{layout::{Layout, Rect}, style::Color, widgets::{canvas::Canvas, Block}};

use crate::{constants::{real::FIELD, DT}, GC};

const FIELD_RATIO: f64 = FIELD.0 / FIELD.1;

pub struct TUI {
    gc: GC,
    start: Instant,
}
impl TUI {
    pub fn run(mut gc: GC) {
        let start = Instant::now();
        let mut terminal = ratatui::init();

        loop {
            if start.elapsed().as_millis_f64() > DT * gc.simu.t as f64 {
                gc.step();
            }
            terminal.draw(|frame| {
                let area = frame.area();
                let zoom = (area.width as f64 /2. / FIELD.0).min(area.height as f64 / FIELD.1);
                frame.render_widget(
                    Canvas::default()
                        .block(Block::bordered())
                        .background_color(Color::Green)
                        .paint(|ctx| {}),
                    Rect::new(0, 0,(FIELD.0*zoom) as u16 * 2, (FIELD.1*zoom) as u16));
            });
        }
    }
}