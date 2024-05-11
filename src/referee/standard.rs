use crate::constants::simu::*;
use crate::game_state::{
    Referee, RefereeTeam, RefereeTeamRobot, RefereeTeamRobots, RefereeTeams, Robot, RobotTask,
};
use std::sync::{Arc, Mutex};

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

pub struct RefereeInfo {
    state: GCState,
    #[cfg(not(target_arch = "wasm32"))]
    pub tasks: Arc<Mutex<[Option<RobotTask>; 4]>>,
    #[cfg(target_arch = "wasm32")]
    pub tasks: Rc<RefCell<[Option<RobotTask>; 4]>>,
    blue_team_positive: bool,
    teams: [GCTeam; 2],
    timer: usize,
}

impl RefereeInfo {
    pub fn new(
        blue_team_name: String,
        green_team_name: String,
        blue_team_key: String,
        green_team_key: String,
        blue_team_positive: bool,
    ) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let tasks = Arc::new(Mutex::new(std::array::from_fn(|_| None)));
        #[cfg(target_arch = "wasm32")]
        let tasks = Rc::new(RefCell::new(std::array::from_fn(|_| None)));
        Self {
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
            blue_team_positive,
            timer: 0,
            state: GCState::Nothing,
            tasks,
        }
        // TODO
    }
    pub fn get_referee_state(&self, t: usize) -> Referee {
        #[cfg(not(target_arch = "wasm32"))]
        let tasks = self.tasks.lock().unwrap();
        #[cfg(target_arch = "wasm32")]
        let tasks = self.tasks.borrow();
        Referee {
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
                                penalized_remaining: if let RobotTask::Penalty { start, .. } = task
                                {
                                    Some(
                                        (start + PENALTY_DURATION).saturating_sub(t)
                                            * FRAME_DURATION
                                            / 1000,
                                    )
                                } else {
                                    None
                                },
                                penalized_reason: if let RobotTask::Penalty { reason, .. } = task {
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
                                penalized_remaining: if let RobotTask::Penalty { start, .. } = task
                                {
                                    Some(
                                        (start + PENALTY_DURATION).saturating_sub(t)
                                            * FRAME_DURATION
                                            / 1000,
                                    )
                                } else {
                                    None
                                },
                                penalized_reason: if let RobotTask::Penalty { reason, .. } = task {
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
                                penalized_remaining: if let RobotTask::Penalty { start, .. } = task
                                {
                                    Some(
                                        (start + PENALTY_DURATION).saturating_sub(t)
                                            * FRAME_DURATION
                                            / 1000,
                                    )
                                } else {
                                    None
                                },
                                penalized_reason: if let RobotTask::Penalty { reason, .. } = task {
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
                                penalized_remaining: if let RobotTask::Penalty { start, .. } = task
                                {
                                    Some(
                                        (start + PENALTY_DURATION).saturating_sub(t)
                                            * FRAME_DURATION
                                            / 1000,
                                    )
                                } else {
                                    None
                                },
                                penalized_reason: if let RobotTask::Penalty { reason, .. } = task {
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
        }
    }
}
