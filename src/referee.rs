use std::cell::RefCell;
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::Mutex;
use crate::{game_state::GameState, simulation::Simulation, GC};
use crate::constants::real::*;
use crate::game_state::{Referee as GSReferee, RefereeTeam, RefereeTeamRobot, RefereeTeamRobots, RefereeTeams, RobotTasks};


#[cfg(not(target_arch = "wasm32"))]
pub type TasksType = Arc<Mutex<[RobotTasks; 4]>>;

#[cfg(target_arch = "wasm32")]
pub type TasksType = Rc<RefCell<[RobotTasks; 4]>>;

#[derive(Debug)]
struct Team {
    name: String,
    key: String,
    score: usize
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PlayState {
    Nothing,
    /// Start in frames
    GameRunning(usize),
    /// Timer before the pause
    GamePaused(usize),
    Halftime,
}
impl From<PlayState> for String {
    fn from(val: PlayState) -> Self {
        match val {
            PlayState::Nothing => "Game is ready to start".to_string(),
            _ => "".to_string(), // TODO
        }
    }
}

pub struct Referee {
    /// [blue, green]
    pub teams: [Team; 2],
    blue_team_positive: bool,
    state: PlayState,
    pub tasks: TasksType
}
impl Referee {
    pub fn new(
        blue_team_name: String,
        green_team_name: String,
        blue_team_key: String,
        green_team_key: String,
        blue_team_positive: bool
    ) -> Self {
        Self {
            teams: [
                Team {
                    name: blue_team_name,
                    key: blue_team_key,
                    score: 0
                },
                Team {
                    name: green_team_name,
                    key: green_team_key,
                    score: 0
                },
            ],
            blue_team_positive,
            state: PlayState::GameRunning(0),
            tasks: TasksType::default()
        }
    }
    pub fn get_gs_referee(&self, t: usize) -> GSReferee {
        #[cfg(not(target_arch = "wasm32"))]
        let tasks = self.tasks.blocking_lock();
        #[cfg(target_arch = "wasm32")]
        let tasks = self.tasks.borrow();
        GSReferee {
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
            game_is_running: if let PlayState::GameRunning(_) = self.state {true} else {false},
            game_paused: if let PlayState::GamePaused(_) = self.state {true} else {false},
            halftime_is_running: self.state == PlayState::Halftime,
            timer: match self.state {
                PlayState::GameRunning(start) => (t - start) * FRAME_DURATION / 1000,
                PlayState::GamePaused(timer) => timer * FRAME_DURATION / 1000,
                PlayState::Halftime => MATCH_DURATION.as_secs() as usize/2,
                PlayState::Nothing => 0
            },
            game_state_msg: self.state.into(),
        }
    }
}
impl GC {
    #[cfg(feature = "referee")]
    pub fn referee_step(&mut self) {
        use tracing::info;

        if let PlayState::GameRunning(_) = self.referee.state {
            let gs = self.get_game_state();
            let ball = gs.ball.unwrap();
            if ball.y.abs() < GOAL_HEIGHT/2. {
                if ball.x < -FIELD.0/2. {
                    self.referee.teams[1].score += 1;
                    self.reset();
                    info!(target:"referee", "Green scored!");
                } else if ball.x > FIELD.0/2. {
                    self.referee.teams[0].score += 1;
                    self.reset();
                    info!(target:"referee", "Blue scored!");
                }
            }
        }
    }
    
}