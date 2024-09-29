//! The main game controller implementation. Runs both nativelly and on wasm

use std::cell::RefCell;
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::Mutex;

use crate::constants::simu::*;
use crate::game_state::{
    GameState, Markers, Pose, RefereeTeam, RefereeTeamRobot, RefereeTeamRobots,
    RefereeTeams, Robot, RobotTasks, Referee as GSReferee
};
use crate::simulation::Simulation;
use crate::referee::Referee;
use rapier2d_f64::prelude::*;
use tracing::info;

#[cfg(feature = "control")]
use crate::Control;

#[cfg(not(target_arch = "wasm32"))]
type TasksType = Arc<Mutex<[RobotTasks; 4]>>;

#[cfg(target_arch = "wasm32")]
type TasksType = Rc<RefCell<[RobotTasks; 4]>>;

/// Game controller
pub struct GC {
    #[cfg(feature = "control")]
    control: Control,
    pub simu: Simulation,
    /// It’s None if game has not started
    pub referee: Referee,
}
impl GC {
    pub fn new(
        blue_team_name: String,
        green_team_name: String,
        blue_team_key: String,
        green_team_key: String,
        blue_team_positive: bool,
        #[cfg(feature = "http_client")]
        session_id: &str
    ) -> Self {
        let simu = Simulation::new();
        let referee = Referee::new(blue_team_name, green_team_name, blue_team_key.clone(), green_team_key.clone(), blue_team_positive);
        Self {
            #[cfg(feature = "control")]
            control: Control::new(
                [blue_team_key, green_team_key],
                referee.tasks.clone(),
                #[cfg(feature = "http_client")]
                session_id
            ),
            simu,
            referee,
        }
    }
    pub fn step(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        let mut tasks = self.referee.tasks.blocking_lock();
        #[cfg(target_arch = "wasm32")]
        let mut tasks = self.referee.tasks.borrow_mut();
        for robot in Robot::all() {
            let (x, y, r) = tasks[robot as usize].control;
            let x = x as f64*MULTIPLIER;
            let y = y as f64*MULTIPLIER;

            let handle = self.get_robot_handle(robot);
            let body = &mut self.simu.bodies[handle];
            let mut speed = vector![x, y].norm();
            if speed > ROBOT_SPEED {
                speed = ROBOT_SPEED;
            }
            let angle = y.atan2(x) + body.rotation().angle();
            let x = angle.cos();
            let y = angle.sin();

            let linvel = vector![x, y] * speed;
            let angvel = (r as f64).min(ROBOT_ANGULAR_SPEED).max(-ROBOT_ANGULAR_SPEED);
            
            body.set_linvel(linvel, true);
            body.set_angvel(angvel, true);
            
            if let Some(f) = tasks[robot as usize].kick {
                info!("{:?} : {}", robot, f);
                self.simu.kick(robot, f as f64);
                tasks[robot as usize].kick = None;
            }
        }
        drop(tasks);
        self.simu.step();
        #[cfg(feature = "referee")]
        self.referee_step();
        #[cfg(feature = "control")]
        self.control.publish(self.get_game_state());
    }
    pub fn get_game_state(&self) -> GameState {
        let robots = Robot::all().map(|r| &self.simu.bodies[self.get_robot_handle(r)]);
        let t = self.simu.t;
        let ball = self.simu.bodies[self.simu.ball].translation();
        GameState {
            ball: Some(point![ball.x/MULTIPLIER, ball.y/MULTIPLIER]),
            markers: Markers {
                blue1: Pose {
                    position: point![
                        robots[Robot::Blue1 as usize].translation().x/MULTIPLIER,
                        robots[Robot::Blue1 as usize].translation().y/MULTIPLIER
                    ],
                    orientation: robots[Robot::Blue1 as usize].rotation().angle(),
                },
                blue2: Pose {
                    position: point![
                        robots[Robot::Blue2 as usize].translation().x/MULTIPLIER,
                        robots[Robot::Blue2 as usize].translation().y/MULTIPLIER
                    ],
                    orientation: robots[Robot::Blue2 as usize].rotation().angle(),
                },
                green1: Pose {
                    position: point![
                        robots[Robot::Green1 as usize].translation().x/MULTIPLIER,
                        robots[Robot::Green1 as usize].translation().y/MULTIPLIER
                    ],
                    orientation: robots[Robot::Green1 as usize].rotation().angle(),
                },
                green2: Pose {
                    position: point![
                        robots[Robot::Green2 as usize].translation().x/MULTIPLIER,
                        robots[Robot::Green2 as usize].translation().y/MULTIPLIER
                    ],
                    orientation: robots[Robot::Green2 as usize].rotation().angle(),
                },
            },
            referee: self.referee.get_gs_referee(self.simu.t)
        }
    }
    pub fn find_entity_at(&mut self, pos: Point<f64>) -> Option<RigidBodyHandle> {
        self.simu.find_entity_at(pos*MULTIPLIER)
    }
    pub fn teleport_entity(&mut self, entity: RigidBodyHandle, pos: Point<f64>, r: Option<f64>) {
        self.simu.teleport_entity(entity, pos*MULTIPLIER, r)
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
    pub fn kick(&mut self, id: Robot, f: f64) {
        self.simu.kick(id, f)
    }
    pub fn get_kicker_pose(&self, id: Robot) -> Pose {
        let pos = self.simu.bodies[self.simu.kickers[id as usize]].position();
        Pose {
            position: Point::new(pos.translation.x/MULTIPLIER, pos.translation.y/MULTIPLIER),
            orientation: pos.rotation.angle()
        }
    }
}
