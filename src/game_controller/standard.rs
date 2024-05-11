use crate::constants::simu::*;
use crate::game_state::{GameState, Markers, Pose, Robot, RobotTask};
use crate::simulation::Simulation;
use rapier2d_f64::prelude::*;

#[cfg(feature = "control")]
use crate::control::Control;
use crate::referee::RefereeInfo;

/// Game controller
pub struct GC {
    #[cfg(feature = "control")]
    control: Control,
    pub simu: Simulation,
    // [blue, green]
    referee: RefereeInfo,
}
impl GC {
    pub fn new(
        blue_team_name: String,
        green_team_name: String,
        blue_team_key: String,
        green_team_key: String,
        blue_team_positive: bool,
    ) -> Self {
        let simu = Simulation::new();
        let referee = RefereeInfo::new(
            blue_team_name,
            green_team_name,
            blue_team_key.clone(),
            green_team_key.clone(),
            blue_team_positive,
        );
        Self {
            #[cfg(feature = "control")]
            control: Control::new(
                [blue_team_key.clone(), green_team_key.clone()],
                referee.tasks.clone(),
            ),
            simu,
            referee,
        }
    }
    pub fn step(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        let tasks = self.referee.tasks.lock().unwrap();
        #[cfg(target_arch = "wasm32")]
        let tasks = self.referee.tasks.borrow();
        // dbg!(&tasks);
        for robot in Robot::all() {
            // let handle = self.get_robot_handle(robot);
            // self.simu.bodies[handle].set_linvel(vector![0., 0.], true);
            match &tasks[robot as usize] {
                Some(RobotTask::Control { x, y, r }) => {
                    dbg!(robot);
                    let x = *x as f64 * MULTIPLIER;
                    let y = *y as f64 * MULTIPLIER;
                    // dbg!(x, y, r);
                    let handle = self.get_robot_handle(robot);
                    let body = &mut self.simu.bodies[handle];
                    let mut speed = vector![x, y].norm();
                    if speed > ROBOT_SPEED {
                        speed = ROBOT_SPEED;
                    }
                    let angle = y.atan2(x) + body.rotation().angle();
                    dbg!(angle);
                    dbg!(vector![x, y]);
                    let x = angle.cos();
                    let y = angle.sin();

                    body.set_linvel(dbg!(vector![x, y] * speed), true);
                    body.set_angvel(
                        (*r as f64)
                            .min(ROBOT_ANGULAR_SPEED)
                            .max(-ROBOT_ANGULAR_SPEED),
                        true,
                    );

                    // body.apply_impulse((vector![x, y] * speed) - body.linvel(), true);
                    // body.apply_torque_impulse((*r as f64).min(ROBOT_ANGULAR_SPEED).max(-ROBOT_ANGULAR_SPEED), true);
                }
                _ => {}
            }
        }
        drop(tasks);
        self.simu.step();
        #[cfg(feature = "control")]
        self.control.publish(self.get_game_state());
    }
    pub fn get_game_state(&self) -> GameState {
        let robots = Robot::all().map(|r| &self.simu.bodies[self.get_robot_handle(r)]);
        let t = self.simu.t;
        let ball = self.simu.bodies[self.simu.ball].translation();
        GameState {
            ball: Some(point![ball.x / MULTIPLIER, ball.y / MULTIPLIER]),
            markers: Markers {
                blue1: Pose {
                    position: point![
                        robots[Robot::Blue1 as usize].translation().x / MULTIPLIER,
                        robots[Robot::Blue1 as usize].translation().y / MULTIPLIER
                    ],
                    orientation: robots[Robot::Blue1 as usize].rotation().angle(),
                },
                blue2: Pose {
                    position: point![
                        robots[Robot::Blue2 as usize].translation().x / MULTIPLIER,
                        robots[Robot::Blue2 as usize].translation().y / MULTIPLIER
                    ],
                    orientation: robots[Robot::Blue2 as usize].rotation().angle(),
                },
                green1: Pose {
                    position: point![
                        robots[Robot::Green1 as usize].translation().x / MULTIPLIER,
                        robots[Robot::Green1 as usize].translation().y / MULTIPLIER
                    ],
                    orientation: robots[Robot::Green1 as usize].rotation().angle(),
                },
                green2: Pose {
                    position: point![
                        robots[Robot::Green2 as usize].translation().x / MULTIPLIER,
                        robots[Robot::Green2 as usize].translation().y / MULTIPLIER
                    ],
                    orientation: robots[Robot::Green2 as usize].rotation().angle(),
                },
            },
            referee: self.referee.get_referee_state(t),
        }
    }
    pub fn find_entity_at(&mut self, pos: Point<f64>) -> Option<RigidBodyHandle> {
        self.simu.find_entity_at(pos * MULTIPLIER)
    }
    pub fn teleport_entity(&mut self, entity: RigidBodyHandle, pos: Point<f64>, r: Option<f64>) {
        self.simu.teleport_entity(entity, pos * MULTIPLIER, r)
    }
    pub fn get_ball_handle(&self) -> RigidBodyHandle {
        self.simu.get_ball_handle()
    }
    pub fn get_robot_handle(&self, id: Robot) -> RigidBodyHandle {
        self.simu.get_robot_handle(id)
    }
    pub fn reset(&mut self) {
        self.simu.reset()
    }
}
