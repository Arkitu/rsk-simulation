use core::f64;
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
use nalgebra::{Rotation2, Vector2};
use rapier2d_f64::math::{Rotation, Vector};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::Mutex;
use crate::{game_state::GameState, simulation::Simulation, GC};
use crate::constants::*;
use crate::game_state::{Referee as GSReferee, RefereeTeam, RefereeTeamRobot, RefereeTeamRobots, RefereeTeams, Robot, RobotTasks};


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
    teams: [Team; 2],
    blue_team_positive: bool,
    state: PlayState,
    pub tasks: TasksType,
    with_ball: [usize; 4]
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
            tasks: TasksType::default(),
            with_ball: [0; 4]
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn lock_tasks(&self) -> tokio::sync::MutexGuard<'_, [RobotTasks; 4]> {
        self.tasks.blocking_lock()
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn lock_tasks_mut(&self) -> tokio::sync::MutexGuard<'_, [RobotTasks; 4]> {
        self.lock_tasks()
    }
    #[cfg(target_arch = "wasm32")]
    pub fn lock_tasks(&self) -> std::cell::Ref<'_, [RobotTasks; 4]> {
        self.tasks.borrow()
    }
    #[cfg(target_arch = "wasm32")]
    pub fn lock_tasks_mut(&self) -> std::cell::RefMut<'_, [RobotTasks; 4]> {
        self.tasks.borrow_mut()
    }
    pub fn get_gs_referee(&self, t: usize) -> GSReferee {
        let tasks = self.lock_tasks();
        GSReferee {
            teams: RefereeTeams {
                blue: RefereeTeam {
                    name: self.teams[0].name.clone(),
                    x_positive: self.blue_team_positive,
                    score: self.teams[0].score,
                    robots: RefereeTeamRobots {
                        one: if let Some((reason, end, _)) = &tasks[0].penalty {
                            RefereeTeamRobot {
                                penalized: true,
                                penalized_remaining: Some(end.saturating_sub(t) * FRAME_DURATION / 1000),
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
                        two: if let Some((reason, end, _)) = &tasks[1].penalty {
                            RefereeTeamRobot {
                                penalized: true,
                                penalized_remaining: Some(end.saturating_sub(t) * FRAME_DURATION / 1000),
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
                        one: if let Some((reason, end, _)) = &tasks[2].penalty {
                            RefereeTeamRobot {
                                penalized: true,
                                penalized_remaining: Some(end.saturating_sub(t) * FRAME_DURATION / 1000),
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
                        two: if let Some((reason, end, _)) = &tasks[3].penalty {
                            RefereeTeamRobot {
                                penalized: true,
                                penalized_remaining: Some(end.saturating_sub(t) * FRAME_DURATION / 1000),
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
    pub fn referee_step(&mut self) {
        use rapier2d_f64::math::Point;
        use tracing::info;

        if let PlayState::GameRunning(_) = self.referee.state {
            let gs = self.get_game_state();
            let mut ball = gs.ball.unwrap();
            // Check for goals
            if ball.y.abs() < real::GOAL_HEIGHT/2. {
                if ball.x < -real::FIELD.0/2. {
                    self.referee.teams[1].score += 1;
                    for t in self.referee.lock_tasks_mut().iter_mut() {
                        t.penalty = None
                    }
                    self.reset();
                    ball = real::DEFAULT_BALL_POS;
                    info!(target:"referee", "Green scored!");
                } else if ball.x > real::FIELD.0/2. {
                    self.referee.teams[0].score += 1;
                    for t in self.referee.lock_tasks_mut().iter_mut() {
                        t.penalty = None
                    }
                    self.reset();
                    ball = real::DEFAULT_BALL_POS;
                    info!(target:"referee", "Blue scored!");
                }
            }
            // Check out of field
            if ball.y.abs() > real::FIELD.1/2. || ball.x.abs() > real::FIELD.0/2. {
                self.teleport_entity(self.simu.ball, Point::new(real::DOT_POS.0*ball.x.signum(), real::DOT_POS.1*ball.y.signum()), None);
                info!(target:"referee", "Ball out of field");
            }
            // Check with ball
            for r in Robot::all() {
                if (self.simu.bodies[self.simu.robots[r as usize]].translation() - self.simu.bodies[self.simu.ball].translation()).norm() > simu::BALL_ABUSE_RADIUS
                || self.referee.lock_tasks()[r as usize].penalty.is_some() {
                    self.referee.with_ball[r as usize] = self.simu.t;
                }
                if self.simu.t - self.referee.with_ball[r as usize] > BALL_ABUSE_TIME {
                    self.penalize(r, "Ball abuse");
                }
            }
            // for ((with_ball, handle), r) in self.referee.with_ball.iter_mut().zip(self.simu.robots).zip(Robot::all()) {
            //     if (self.simu.bodies[handle].translation() - self.simu.bodies[self.simu.ball].translation()).norm() > simu::BALL_ABUSE_RADIUS {
            //         *with_ball = self.simu.t;
            //     }
            //     if self.simu.t - *with_ball > BALL_ABUSE_TIME {
            //         self.penalize(r, "Ball abuse");
            //     }
            // }
            for (t, r) in self.referee.lock_tasks_mut().iter_mut().zip(Robot::all()) {
                if let Some((_, end, spot)) = t.penalty {
                    if end < self.simu.t {
                        t.penalty = None;
                        t.control = (0., 0., 0.);
                    } else {
                        // goto DIY
                        // TODO: maybe make it a copy of the official goto
                        
                        let r_pos = &self.simu.bodies[self.simu.robots[r as usize]];
                        let spot_pos = simu::PENALTY_SPOTS[spot];

                        let spot_ang = if spot < 3 {
                            f64::consts::FRAC_PI_2
                        } else {
                            -f64::consts::FRAC_PI_2
                        };
                        let angle = r_pos.rotation().rotation_to(&Rotation::new(spot_ang)).angle();
                        let ang_d = (angle+f64::consts::PI).sqrt();
                        let ang_speed = (ang_d).min(simu::ROBOT_ANGULAR_SPEED);

                        
                        let vec = Rotation2::new(-r_pos.rotation().angle()) * (spot_pos.coords - r_pos.translation());
                        
                        let d = vec.norm();
                        let speed = (d*0.3).min(simu::ROBOT_SPEED);
                        
                        t.control = ((vec.normalize().x*speed) as f32, (vec.normalize().y*speed) as f32, (angle.signum()*ang_speed) as f32);
                    }
                }
            }
        }
    }
    pub fn penalize(&self, r: Robot, reason: &'static str) {
        dbg!("penalize");
        let mut tasks = self.referee.lock_tasks_mut();

        if let Some(p) = tasks[r as usize].penalty.as_mut() {
            p.0 = reason;
            p.1 += PENALTY_DURATION;
            return
        }

        let r_pos = self.simu.bodies[self.simu.robots[r as usize]].translation();
        
        let spot = simu::PENALTY_SPOTS.into_iter()
            .enumerate()
            .filter(|(i, p)| {
                !(
                    self.simu.robots.iter().enumerate().filter(|(i,_)| *i != r as usize).any(|(_,r)| {
                        (self.simu.bodies.get(*r).unwrap().translation()-p.coords).norm()<simu::ROBOT_RADIUS
                    })
                    || tasks.iter().any(|t|
                        match t.penalty {
                            None => false,
                            Some((_, _, s)) => s == *i
                        }
                    )
                )
            })
            .map(|(i, p)| {
                (i, p, (p.coords-r_pos).norm())
            })
            .max_by(|a, b| {
                b.2.total_cmp(&a.2)
            })
            .map(|(i, _, _)| i)
            .unwrap_or(0);
        
        tasks[r as usize].penalty = Some((reason, self.simu.t+PENALTY_DURATION, spot));
    }
}