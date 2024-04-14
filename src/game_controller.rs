use rapier2d::prelude::*;
use crate::{game_state::{GameState, Markers, Pose, Referee, RefereeTeam, RefereeTeamRobot, RefereeTeamRobots, RefereeTeams}, DT};

const FRAME_DURATION: usize = (DT*1000.) as usize; // in ms
const PENALTY_DURATION: usize = 5000 / FRAME_DURATION; // in frames

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum GCState {
    Nothing,
    GameRunning,
    GamePaused,
    Halftime
}
impl Into<&'static str> for GCState {
    fn into(self) -> &'static str {
        match self {
            GCState::Nothing => "Game is ready to start",
            _ => "" // TODO
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum RobotTask {
    // TODO
}
impl Into<&'static str> for RobotTask {
    fn into(self) -> &'static str {
        match self {
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PenaltyReason {
    // TODO
}
impl Into<&'static str> for PenaltyReason {
    fn into(self) -> &'static str {
        match self {
        }
    }
}

#[derive(Debug)]
struct GCRobot {
    tasks: Vec<RobotTask>,
    penalty_reason: Option<PenaltyReason>,
    // Frame number when the penalty started
    penalty_start: usize,
    handle: RigidBodyHandle
}

#[derive(Debug)]
struct GCTeam {
    name: &'static str,
    score: usize,
    robots: [GCRobot; 2]
}

#[derive(Debug)]
pub struct GC {
    state: GCState,
    ball: RigidBodyHandle,
    // [blue, green]
    teams: [GCTeam; 2],
    blue_team_positive: bool,
    timer: usize
}
impl GC {
    pub fn new(
        ball: RigidBodyHandle,
        robots: [RigidBodyHandle; 4],
        blue_team_name: &'static str,
        green_team_name: &'static str,
        blue_team_positive: bool
    ) -> Self {
        Self {
            state: GCState::Nothing,
            ball,
            teams: [
                GCTeam {
                    name: blue_team_name,
                    score: 0,
                    robots: [
                        GCRobot {
                            tasks: vec![],
                            penalty_reason: None,
                            handle: robots[0],
                            penalty_start: 0
                        },
                        GCRobot {
                            tasks: vec![],
                            penalty_reason: None,
                            handle: robots[1],
                            penalty_start: 0
                        }
                    ]
                },
                GCTeam {
                    name: green_team_name,
                    score: 0,
                    robots: [
                        GCRobot {
                            tasks: vec![],
                            penalty_reason: None,
                            handle: robots[2],
                            penalty_start: 0
                        },
                        GCRobot {
                            tasks: vec![],
                            penalty_reason: None,
                            handle: robots[3],
                            penalty_start: 0
                        }
                    ]
                }
            ],
            blue_team_positive,
            timer: 0
        }
    }
    pub fn get_game_state(&self, bodies: &RigidBodySet, current_frame: usize) -> GameState {
        let ball = bodies[self.ball].translation();
        let mut robots = self.teams.iter().flat_map(|t| t.robots.iter());
        let robots = [robots.next().unwrap(), robots.next().unwrap(), robots.next().unwrap(), robots.next().unwrap()];
        GameState {
            ball: point![ball.x, ball.y],
            markers: Markers {
                blue1: Pose {
                    position: point![bodies[robots[0].handle].translation().x, bodies[robots[0].handle].translation().y],
                    orientation: bodies[robots[0].handle].rotation().angle()
                },
                blue2: Pose {
                    position: point![bodies[robots[1].handle].translation().x, bodies[robots[1].handle].translation().y],
                    orientation: bodies[robots[1].handle].rotation().angle()
                },
                green1: Pose {
                    position: point![bodies[robots[2].handle].translation().x, bodies[robots[2].handle].translation().y],
                    orientation: bodies[robots[2].handle].rotation().angle()
                },
                green2: Pose {
                    position: point![bodies[robots[3].handle].translation().x, bodies[robots[3].handle].translation().y],
                    orientation: bodies[robots[3].handle].rotation().angle()
                }
            },
            referee: Referee {
                teams: RefereeTeams {
                    blue: RefereeTeam {
                        name: self.teams[0].name,
                        x_positive: self.blue_team_positive,
                        score: self.teams[0].score,
                        robots: RefereeTeamRobots {
                            one: RefereeTeamRobot {
                                penalized: self.teams[0].robots[0].penalty_reason.is_some(),
                                penalized_remaining: self.teams[0].robots[0].penalty_reason.map(|_| (self.teams[0].robots[0].penalty_start + PENALTY_DURATION).saturating_sub(current_frame)*FRAME_DURATION/1000),
                                penalized_reson: self.teams[0].robots[0].penalty_reason.map(|pr| pr.into()),
                                preempted: self.teams[0].robots[0].tasks.len() > 1,
                                preemption_reasons: self.teams[0].robots[0].tasks.iter().map(|t| (*t).into()).collect()
                            },
                            two: RefereeTeamRobot {
                                penalized: self.teams[0].robots[1].penalty_reason.is_some(),
                                penalized_remaining: self.teams[0].robots[1].penalty_reason.map(|_| (self.teams[0].robots[1].penalty_start + PENALTY_DURATION).saturating_sub(current_frame)*FRAME_DURATION/1000),
                                penalized_reson: self.teams[0].robots[1].penalty_reason.map(|pr| pr.into()),
                                preempted: self.teams[0].robots[1].tasks.len() > 1,
                                preemption_reasons: self.teams[0].robots[1].tasks.iter().map(|t| (*t).into()).collect()
                            }
                        }
                    },
                    green: RefereeTeam {
                        name: self.teams[1].name,
                        x_positive: !self.blue_team_positive,
                        score: self.teams[1].score,
                        robots: RefereeTeamRobots {
                            one: RefereeTeamRobot {
                                penalized: self.teams[1].robots[0].penalty_reason.is_some(),
                                penalized_remaining: self.teams[1].robots[0].penalty_reason.map(|_| (self.teams[1].robots[0].penalty_start + PENALTY_DURATION).saturating_sub(current_frame)*FRAME_DURATION/1000),
                                penalized_reson: self.teams[1].robots[0].penalty_reason.map(|pr| pr.into()),
                                preempted: self.teams[1].robots[0].tasks.len() > 1,
                                preemption_reasons: self.teams[1].robots[0].tasks.iter().map(|t| (*t).into()).collect()
                            },
                            two: RefereeTeamRobot {
                                penalized: self.teams[1].robots[1].penalty_reason.is_some(),
                                penalized_remaining: self.teams[1].robots[1].penalty_reason.map(|_| (self.teams[1].robots[1].penalty_start + PENALTY_DURATION).saturating_sub(current_frame)*FRAME_DURATION/1000),
                                penalized_reson: self.teams[1].robots[1].penalty_reason.map(|pr| pr.into()),
                                preempted: self.teams[1].robots[1].tasks.len() > 1,
                                preemption_reasons: self.teams[1].robots[1].tasks.iter().map(|t| (*t).into()).collect()
                            }
                        }
                    }
                },
                game_is_running: self.state == GCState::GameRunning,
                game_paused: self.state == GCState::GamePaused,
                halftime_is_running: self.state == GCState::Halftime,
                timer: self.timer * FRAME_DURATION / 1000,
                game_state_msg: self.state.into()
            }
        }
    }
}