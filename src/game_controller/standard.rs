use crate::constants::*;
use crate::game_state::{
    GameState, Markers, Pose, Referee, RefereeTeam, RefereeTeamRobot, RefereeTeamRobots,
    RefereeTeams, Robot,
};
use crate::simulation::Simulation;
use rapier2d::prelude::*;

use super::GCTrait;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum GCState {
    Nothing,
    GameRunning,
    GamePaused,
    Halftime,
}
impl From<GCState> for &'static str {
    fn from(val: GCState) -> Self {
        match val {
            GCState::Nothing => "Game is ready to start",
            _ => "", // TODO
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum RobotTask {
    // TODO
}
impl From<RobotTask> for &'static str {
    fn from(val: RobotTask) -> Self {
        match val {}
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PenaltyReason {
    // TODO
}
impl From<PenaltyReason> for &'static str {
    fn from(val: PenaltyReason) -> Self {
        match val {}
    }
}

#[derive(Debug)]
struct GCRobot {
    tasks: Vec<RobotTask>,
    penalty_reason: Option<PenaltyReason>,
    // Frame number when the penalty started
    penalty_start: usize,
    handle: RigidBodyHandle,
}

#[derive(Debug)]
struct GCTeam {
    name: &'static str,
    score: usize,
    robots: [GCRobot; 2],
}

/// Game controller
pub struct GC {
    simu: Simulation,
    state: GCState,
    // [blue, green]
    teams: [GCTeam; 2],
    blue_team_positive: bool,
    timer: usize,
}
impl GCTrait for GC {
    fn new(
        blue_team_name: &'static str,
        green_team_name: &'static str,
        blue_team_positive: bool,
    ) -> Self {
        let simu = Simulation::new();
        let robots = simu.robots;
        Self {
            simu,
            state: GCState::Nothing,
            teams: [
                GCTeam {
                    name: blue_team_name,
                    score: 0,
                    robots: [
                        GCRobot {
                            tasks: vec![],
                            penalty_reason: None,
                            handle: robots[0],
                            penalty_start: 0,
                        },
                        GCRobot {
                            tasks: vec![],
                            penalty_reason: None,
                            handle: robots[1],
                            penalty_start: 0,
                        },
                    ],
                },
                GCTeam {
                    name: green_team_name,
                    score: 0,
                    robots: [
                        GCRobot {
                            tasks: vec![],
                            penalty_reason: None,
                            handle: robots[2],
                            penalty_start: 0,
                        },
                        GCRobot {
                            tasks: vec![],
                            penalty_reason: None,
                            handle: robots[3],
                            penalty_start: 0,
                        },
                    ],
                },
            ],
            blue_team_positive,
            timer: 0,
        }
    }
    fn step(&mut self) {
        self.simu.step();
    }
    fn get_game_state(&self) -> GameState {
        let bodies = &self.simu.bodies;
        let t = self.simu.t;
        let ball = bodies[self.simu.ball].translation();
        let mut robots = self.teams.iter().flat_map(|t| t.robots.iter());
        let robots = [
            robots.next().unwrap(),
            robots.next().unwrap(),
            robots.next().unwrap(),
            robots.next().unwrap(),
        ];
        GameState {
            ball: Some(point![ball.x, ball.y]),
            markers: Markers {
                blue1: Pose {
                    position: point![
                        bodies[robots[0].handle].translation().x,
                        bodies[robots[0].handle].translation().y
                    ],
                    orientation: bodies[robots[0].handle].rotation().angle(),
                },
                blue2: Pose {
                    position: point![
                        bodies[robots[1].handle].translation().x,
                        bodies[robots[1].handle].translation().y
                    ],
                    orientation: bodies[robots[1].handle].rotation().angle(),
                },
                green1: Pose {
                    position: point![
                        bodies[robots[2].handle].translation().x,
                        bodies[robots[2].handle].translation().y
                    ],
                    orientation: bodies[robots[2].handle].rotation().angle(),
                },
                green2: Pose {
                    position: point![
                        bodies[robots[3].handle].translation().x,
                        bodies[robots[3].handle].translation().y
                    ],
                    orientation: bodies[robots[3].handle].rotation().angle(),
                },
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
                                penalized_remaining: self.teams[0].robots[0].penalty_reason.map(
                                    |_| {
                                        (self.teams[0].robots[0].penalty_start + PENALTY_DURATION)
                                            .saturating_sub(t)
                                            * FRAME_DURATION
                                            / 1000
                                    },
                                ),
                                penalized_reson: self.teams[0].robots[0]
                                    .penalty_reason
                                    .map(|pr| pr.into()),
                                preempted: self.teams[0].robots[0].tasks.len() > 1,
                                preemption_reasons: self.teams[0].robots[0]
                                    .tasks
                                    .iter()
                                    .map(|t| (*t).into())
                                    .collect(),
                            },
                            two: RefereeTeamRobot {
                                penalized: self.teams[0].robots[1].penalty_reason.is_some(),
                                penalized_remaining: self.teams[0].robots[1].penalty_reason.map(
                                    |_| {
                                        (self.teams[0].robots[1].penalty_start + PENALTY_DURATION)
                                            .saturating_sub(t)
                                            * FRAME_DURATION
                                            / 1000
                                    },
                                ),
                                penalized_reson: self.teams[0].robots[1]
                                    .penalty_reason
                                    .map(|pr| pr.into()),
                                preempted: self.teams[0].robots[1].tasks.len() > 1,
                                preemption_reasons: self.teams[0].robots[1]
                                    .tasks
                                    .iter()
                                    .map(|t| (*t).into())
                                    .collect(),
                            },
                        },
                    },
                    green: RefereeTeam {
                        name: self.teams[1].name,
                        x_positive: !self.blue_team_positive,
                        score: self.teams[1].score,
                        robots: RefereeTeamRobots {
                            one: RefereeTeamRobot {
                                penalized: self.teams[1].robots[0].penalty_reason.is_some(),
                                penalized_remaining: self.teams[1].robots[0].penalty_reason.map(
                                    |_| {
                                        (self.teams[1].robots[0].penalty_start + PENALTY_DURATION)
                                            .saturating_sub(t)
                                            * FRAME_DURATION
                                            / 1000
                                    },
                                ),
                                penalized_reson: self.teams[1].robots[0]
                                    .penalty_reason
                                    .map(|pr| pr.into()),
                                preempted: self.teams[1].robots[0].tasks.len() > 1,
                                preemption_reasons: self.teams[1].robots[0]
                                    .tasks
                                    .iter()
                                    .map(|t| (*t).into())
                                    .collect(),
                            },
                            two: RefereeTeamRobot {
                                penalized: self.teams[1].robots[1].penalty_reason.is_some(),
                                penalized_remaining: self.teams[1].robots[1].penalty_reason.map(
                                    |_| {
                                        (self.teams[1].robots[1].penalty_start + PENALTY_DURATION)
                                            .saturating_sub(t)
                                            * FRAME_DURATION
                                            / 1000
                                    },
                                ),
                                penalized_reson: self.teams[1].robots[1]
                                    .penalty_reason
                                    .map(|pr| pr.into()),
                                preempted: self.teams[1].robots[1].tasks.len() > 1,
                                preemption_reasons: self.teams[1].robots[1]
                                    .tasks
                                    .iter()
                                    .map(|t| (*t).into())
                                    .collect(),
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
    fn teleport_ball(&mut self, pos: Point<f32>) {
        self.simu.bodies[self.simu.ball].set_position(pos.into(), true);
    }
    fn teleport_robot(&mut self, id: Robot, pos: Point<f32>) {
        let (team, robot) = match id {
            Robot::Blue1 => (0, 0),
            Robot::Blue2 => (0, 1),
            Robot::Green1 => (1, 0),
            Robot::Green2 => (0, 1),
        };
        self.simu.bodies[self.teams[team].robots[robot].handle].set_position(pos.into(), true);
    }
    fn find_entity_at(&self, pos: Point<f32>) -> Option<RigidBodyHandle> {
        self.simu.find_entity_at(pos)
    }
    fn move_entity(&mut self, entity: RigidBodyHandle, pos: Point<f32>) {
        self.simu.bodies[entity].set_position(pos.into(), true);
    }
    fn get_ball_handle(&self) -> RigidBodyHandle {
        self.simu.ball
    }
}
