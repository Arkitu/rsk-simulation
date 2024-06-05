use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use maybe_async::maybe_async;
use tokio::sync::Mutex;

use crate::constants::simu::*;
use crate::game_state::{
    GameState, Markers, Pose, Referee, RefereeTeam, RefereeTeamRobot, RefereeTeamRobots,
    RefereeTeams, Robot, RobotTasks,
};
use crate::simulation::Simulation;
use rapier2d_f64::prelude::*;
use tracing::info;

#[cfg(feature = "control")]
use crate::control::Control;

#[cfg(not(target_arch = "wasm32"))]
type TasksType = Arc<Mutex<[RobotTasks; 4]>>;

#[cfg(target_arch = "wasm32")]
type TasksType = Rc<RefCell<[RobotTasks; 4]>>;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum GCState {
    Nothing,
    GameRunning,
    GamePaused,
    Halftime,
}
impl From<GCState> for String {
    fn from(val: GCState) -> Self {
        match val {
            GCState::Nothing => "Game is ready to start".to_string(),
            _ => "".to_string(), // TODO
        }
    }
}

#[derive(Debug)]
struct GCTeam {
    name: String,
    key: String,
    score: usize
}

/// Game controller
pub struct GC {
    #[cfg(feature = "control")]
    control: Control,
    pub simu: Simulation,
    state: GCState,
    // [blue, green]
    teams: [GCTeam; 2],
    tasks: TasksType,
    blue_team_positive: bool,
    timer: usize,
}
impl GC {
    pub fn new(
        blue_team_name: String,
        green_team_name: String,
        blue_team_key: String,
        green_team_key: String,
        blue_team_positive: bool,
        #[cfg(feature = "http_client_control")]
        session_id: &str
    ) -> Self {
        let simu = Simulation::new();
        let tasks = TasksType::default();
        Self {
            #[cfg(feature = "control")]
            control: Control::new(
                [blue_team_key.clone(), green_team_key.clone()],
                tasks.clone(),
                #[cfg(feature = "http_client_control")]
                session_id
            ),
            simu,
            state: GCState::Nothing,
            teams: [
                GCTeam {
                    name: blue_team_name,
                    key: blue_team_key,
                    score: 0
                },
                GCTeam {
                    name: green_team_name,
                    key: green_team_key,
                    score: 0
                },
            ],
            tasks,
            blue_team_positive,
            timer: 0,
        }
    }
    #[maybe_async]
    pub async fn step(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        let mut tasks = self.tasks.lock().await;
        #[cfg(target_arch = "wasm32")]
        let mut tasks = self.tasks.borrow_mut();
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
        #[cfg(feature = "control")]
        self.control.publish(self.get_game_state().await);
    }
    #[maybe_async]
    pub async fn get_game_state(&self) -> GameState {
        let robots = Robot::all().map(|r| &self.simu.bodies[self.get_robot_handle(r)]);
        let t = self.simu.t;
        let ball = self.simu.bodies[self.simu.ball].translation();
        #[cfg(not(target_arch = "wasm32"))]
        let tasks = self.tasks.lock().await;
        #[cfg(target_arch = "wasm32")]
        let tasks = self.tasks.borrow();
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
            referee: Referee {
                teams: RefereeTeams {
                    blue: RefereeTeam {
                        name: self.teams[0].name.clone(),
                        x_positive: self.blue_team_positive,
                        score: self.teams[0].score,
                        robots: RefereeTeamRobots {
                            one: if let Some((reason, start)) = &tasks[0].penalty {
                                RefereeTeamRobot {
                                    penalized: true,
                                    penalized_remaining: Some((start+PENALTY_DURATION).saturating_sub(t) * FRAME_DURATION / 1000),
                                    penalized_reason: Some(reason.to_string()),
                                    preempted: true,
                                    preemption_reasons: vec![reason.to_string()]
                                }
                            } else {
                                RefereeTeamRobot {
                                    penalized: false,
                                    penalized_remaining: None,
                                    penalized_reason: None,
                                    preempted: false,
                                    preemption_reasons: vec![]
                                }
                            },
                            two: if let Some((reason, start)) = &tasks[1].penalty {
                                RefereeTeamRobot {
                                    penalized: true,
                                    penalized_remaining: Some((start+PENALTY_DURATION).saturating_sub(t) * FRAME_DURATION / 1000),
                                    penalized_reason: Some(reason.to_string()),
                                    preempted: true,
                                    preemption_reasons: vec![reason.to_string()]
                                }
                            } else {
                                RefereeTeamRobot {
                                    penalized: false,
                                    penalized_remaining: None,
                                    penalized_reason: None,
                                    preempted: false,
                                    preemption_reasons: vec![]
                                }
                            },
                        },
                    },
                    green: RefereeTeam {
                        name: self.teams[1].name.clone(),
                        x_positive: !self.blue_team_positive,
                        score: self.teams[1].score,
                        robots: RefereeTeamRobots {
                            one: if let Some((reason, start)) = &tasks[2].penalty {
                                RefereeTeamRobot {
                                    penalized: true,
                                    penalized_remaining: Some((start+PENALTY_DURATION).saturating_sub(t) * FRAME_DURATION / 1000),
                                    penalized_reason: Some(reason.to_string()),
                                    preempted: true,
                                    preemption_reasons: vec![reason.to_string()]
                                }
                            } else {
                                RefereeTeamRobot {
                                    penalized: false,
                                    penalized_remaining: None,
                                    penalized_reason: None,
                                    preempted: false,
                                    preemption_reasons: vec![]
                                }
                            },
                            two: if let Some((reason, start)) = &tasks[3].penalty {
                                RefereeTeamRobot {
                                    penalized: true,
                                    penalized_remaining: Some((start+PENALTY_DURATION).saturating_sub(t) * FRAME_DURATION / 1000),
                                    penalized_reason: Some(reason.to_string()),
                                    preempted: true,
                                    preemption_reasons: vec![reason.to_string()]
                                }
                            } else {
                                RefereeTeamRobot {
                                    penalized: false,
                                    penalized_remaining: None,
                                    penalized_reason: None,
                                    preempted: false,
                                    preemption_reasons: vec![]
                                }
                            },
                        },
                    },
                },
                game_is_running: self.state == GCState::GameRunning,
                game_paused: self.state == GCState::GamePaused,
                halftime_is_running: self.state == GCState::Halftime,
                timer: self.timer * FRAME_DURATION / 1000,
                game_state_msg: self.state.into(),
            },
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
