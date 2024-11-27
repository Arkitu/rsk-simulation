use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Layout, Rect},
    style::Color,
    widgets::{
        canvas::{Canvas, Circle},
        Block,
    },
};
use std::time::{Duration, Instant};

use crate::{
    constants::{real::FIELD, DT, FRAME_DURATION},
    GC,
};

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
            while start.elapsed().as_millis() as usize > FRAME_DURATION * gc.simu.t {
                gc.step();
            }
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    let zoom = (area.width as f64 / 2. / FIELD.0).min(area.height as f64 / FIELD.1);
                    frame.render_widget(
                        Canvas::default()
                            .background_color(Color::Green)
                            .paint(|ctx| {
                                let gs = gc.get_game_state();
                                if let Some(ball) = gs.ball {
                                    ctx.draw(&Circle {
                                        x: ball.x,
                                        y: ball.y,
                                        radius: 1.,
                                        color: Color::Indexed(0),
                                    });
                                }
                            }),
                        Rect::new(0, 0, (FIELD.0 * zoom) as u16 * 2, (FIELD.1 * zoom) as u16),
                    );
                }).unwrap();
            if event::poll(Duration::ZERO).unwrap() {
                match event::read().unwrap() {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}
