use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Layout, Rect},
    style::Color,
    widgets::{
        canvas::{Canvas, Circle, Line},
        Block,
    },
};
use core::f64;
use std::time::{Duration, Instant};

use crate::{
    constants::{real::{BALL_RADIUS, CARPET, FIELD, ROBOT_RADIUS}, DT, FRAME_DURATION}, game_state::Robot, GC
};

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
                    let zoom = (area.width as f64 / 2. / CARPET.0).min(area.height as f64 / CARPET.1);
                    frame.render_widget(
                        Canvas::default()
                            //.background_color(Color::Green)
                            .paint(|ctx| {
                                let gs = gc.get_game_state();
                                if let Some(ball) = gs.ball {
                                    ctx.draw(&Circle {
                                        x: ball.x,
                                        y: ball.y,
                                        radius: BALL_RADIUS,
                                        color: Color::Indexed(202),
                                    });
                                }
                                for (i, r) in [gs.markers.blue1, gs.markers.blue2, gs.markers.green1, gs.markers.green2].into_iter().enumerate() {
                                    ctx.draw(&Circle {
                                        x: r.position.x,
                                        y: r.position.y,
                                        radius: ROBOT_RADIUS,
                                        color: if i < 2 {Color::Blue} else {Color::Green},
                                    });
                                    let kicker = gc.get_kicker_pose(Robot::all()[i]);
                                    ctx.draw(&Line {
                                        x1: kicker.position.x + (ROBOT_RADIUS/2.*(f64::consts::FRAC_PI_2+r.orientation).cos()),
                                        y1: kicker.position.y + (ROBOT_RADIUS/2.*(f64::consts::FRAC_PI_2+r.orientation).sin()),
                                        x2: kicker.position.x + (ROBOT_RADIUS/2.*(-f64::consts::FRAC_PI_2+r.orientation).cos()),
                                        y2: kicker.position.y + (ROBOT_RADIUS/2.*(-f64::consts::FRAC_PI_2+r.orientation).sin()),
                                        color: Color::Gray
                                    });
                                }
                            })
                            //.marker()
                            .x_bounds([-CARPET.0/2., CARPET.0/2.])
                            .y_bounds([-CARPET.1/2., CARPET.1/2.]),
                        Rect::new(0, 0, (CARPET.0 * zoom) as u16 * 2, (CARPET.1 * zoom) as u16),
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
