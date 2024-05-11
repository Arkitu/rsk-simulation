use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::constants::simu::*;
use crate::game_state::{
    GameState, Markers, Pose, Referee, RefereeTeam, RefereeTeamRobot, RefereeTeamRobots,
    RefereeTeams, Robot, RobotTask,
};
use crate::simulation::Simulation;
use rapier2d_f64::prelude::*;

#[cfg(feature = "control")]
use crate::control::Control;

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
    score: usize,
}

/// Game controller
pub struct GC {
    #[cfg(feature = "control")]
    control: Control,
    pub simu: Simulation,
    state: GCState,
    // [blue, green]
    teams: [GCTeam; 2],
    #[cfg(not(target_arch = "wasm32"))]
    tasks: Arc<Mutex<[Option<RobotTask>; 4]>>,
    #[cfg(target_arch = "wasm32")]
    tasks: Rc<RefCell<[Option<RobotTask>; 4]>>,
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
    ) -> Self {
        let simu = Simulation::new();
        #[cfg(not(target_arch = "wasm32"))]
        let tasks = Arc::new(Mutex::new(std::array::from_fn(|_| None)));
        #[cfg(target_arch = "wasm32")]
        let tasks = Rc::new(RefCell::new(std::array::from_fn(|_| None)));
        Self {
            #[cfg(feature = "control")]
            control: Control::new(
                [blue_team_key.clone(), green_team_key.clone()],
                tasks.clone(),
            ),
            simu,
            state: GCState::Nothing,
            teams: [
                GCTeam {
                    name: blue_team_name,
                    key: blue_team_key,
                    score: 0,
                },
                GCTeam {
                    name: green_team_name,
                    key: green_team_key,
                    score: 0,
                },
            ],
            tasks,
            blue_team_positive,
            timer: 0,
        }
    }
    pub fn step(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        let tasks = self.tasks.lock().unwrap();
        #[cfg(target_arch = "wasm32")]
        let tasks = self.tasks.borrow();
        // dbg!(&tasks);
        for robot in Robot::all() {
            let handle = self.get_robot_handle(robot);
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
        #[cfg(not(target_arch = "wasm32"))]
        let tasks = self.tasks.lock().unwrap();
        #[cfg(target_arch = "wasm32")]
        let tasks = self.tasks.borrow();
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
            referee: Referee {
                teams: RefereeTeams {
                    blue: RefereeTeam {
                        name: self.teams[0].name.clone(),
                        x_positive: self.blue_team_positive,
                        score: self.teams[0].score,
                        robots: RefereeTeamRobots {
                            one: if let Some(task) = &tasks[0] {
                                RefereeTeamRobot {
                                    penalized: if let RobotTask::Penalty { .. } = task {
                                        true
                                    } else {
                                        false
                                    },
                                    penalized_remaining: if let RobotTask::Penalty {
                                        start, ..
                                    } = task
                                    {
                                        Some(
                                            (start + PENALTY_DURATION).saturating_sub(t)
                                                * FRAME_DURATION
                                                / 1000,
                                        )
                                    } else {
                                        None
                                    },
                                    penalized_reason: if let RobotTask::Penalty { reason, .. } =
                                        task
                                    {
                                        Some(reason.to_string())
                                    } else {
                                        None
                                    },
                                    preempted: task.preemption_reason(Robot::Blue1).is_some(),
                                    preemption_reasons: task
                                        .preemption_reason(Robot::Blue1)
                                        .map(|r| vec![r])
                                        .unwrap_or(vec![]),
                                }
                            } else {
                                RefereeTeamRobot {
                                    penalized: false,
                                    penalized_remaining: None,
                                    penalized_reason: None,
                                    preempted: false,
                                    preemption_reasons: vec![],
                                }
                            },
                            two: if let Some(task) = &tasks[1] {
                                RefereeTeamRobot {
                                    penalized: if let RobotTask::Penalty { .. } = task {
                                        true
                                    } else {
                                        false
                                    },
                                    penalized_remaining: if let RobotTask::Penalty {
                                        start, ..
                                    } = task
                                    {
                                        Some(
                                            (start + PENALTY_DURATION).saturating_sub(t)
                                                * FRAME_DURATION
                                                / 1000,
                                        )
                                    } else {
                                        None
                                    },
                                    penalized_reason: if let RobotTask::Penalty { reason, .. } =
                                        task
                                    {
                                        Some(reason.to_string())
                                    } else {
                                        None
                                    },
                                    preempted: task.preemption_reason(Robot::Blue2).is_some(),
                                    preemption_reasons: task
                                        .preemption_reason(Robot::Blue2)
                                        .map(|r| vec![r])
                                        .unwrap_or(vec![]),
                                }
                            } else {
                                RefereeTeamRobot {
                                    penalized: false,
                                    penalized_remaining: None,
                                    penalized_reason: None,
                                    preempted: false,
                                    preemption_reasons: vec![],
                                }
                            },
                        },
                    },
                    green: RefereeTeam {
                        name: self.teams[1].name.clone(),
                        x_positive: !self.blue_team_positive,
                        score: self.teams[1].score,
                        robots: RefereeTeamRobots {
                            one: if let Some(task) = &tasks[2] {
                                RefereeTeamRobot {
                                    penalized: if let RobotTask::Penalty { .. } = task {
                                        true
                                    } else {
                                        false
                                    },
                                    penalized_remaining: if let RobotTask::Penalty {
                                        start, ..
                                    } = task
                                    {
                                        Some(
                                            (start + PENALTY_DURATION).saturating_sub(t)
                                                * FRAME_DURATION
                                                / 1000,
                                        )
                                    } else {
                                        None
                                    },
                                    penalized_reason: if let RobotTask::Penalty { reason, .. } =
                                        task
                                    {
                                        Some(reason.to_string())
                                    } else {
                                        None
                                    },
                                    preempted: task.preemption_reason(Robot::Green1).is_some(),
                                    preemption_reasons: task
                                        .preemption_reason(Robot::Green1)
                                        .map(|r| vec![r])
                                        .unwrap_or(vec![]),
                                }
                            } else {
                                RefereeTeamRobot {
                                    penalized: false,
                                    penalized_remaining: None,
                                    penalized_reason: None,
                                    preempted: false,
                                    preemption_reasons: vec![],
                                }
                            },
                            two: if let Some(task) = &tasks[3] {
                                RefereeTeamRobot {
                                    penalized: if let RobotTask::Penalty { .. } = task {
                                        true
                                    } else {
                                        false
                                    },
                                    penalized_remaining: if let RobotTask::Penalty {
                                        start, ..
                                    } = task
                                    {
                                        Some(
                                            (start + PENALTY_DURATION).saturating_sub(t)
                                                * FRAME_DURATION
                                                / 1000,
                                        )
                                    } else {
                                        None
                                    },
                                    penalized_reason: if let RobotTask::Penalty { reason, .. } =
                                        task
                                    {
                                        Some(reason.to_string())
                                    } else {
                                        None
                                    },
                                    preempted: task.preemption_reason(Robot::Green2).is_some(),
                                    preemption_reasons: task
                                        .preemption_reason(Robot::Green2)
                                        .map(|r| vec![r])
                                        .unwrap_or(vec![]),
                                }
                            } else {
                                RefereeTeamRobot {
                                    penalized: false,
                                    penalized_remaining: None,
                                    penalized_reason: None,
                                    preempted: false,
                                    preemption_reasons: vec![],
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
