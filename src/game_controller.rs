use rapier2d::prelude::*;
use crate::game_state::{GameState, Markers, Pose, Referee, RefereeTeam, RefereeTeamRobots, RefereeTeams, RefereeTeamRobot};

enum GCState {
    Nothing,
    GameRunning,
    GamePaused,
}

enum Side {
    Blue,
    Green
}

enum RobotTask {
    // TODO
}

enum PenaltyReason {

}

struct GCRobot {
    tasks: Vec<RobotTask>,
    penalty_reason: Option<PenaltyReason>,

}

struct GCTeam {
    side: Side,
    name: &'static str,
    score: usize,
    robots: 
}

pub struct GC {
    state: GCState,
    ball: RigidBodyHandle,
    blue1: RigidBodyHandle,
    blue2: RigidBodyHandle,
    green1: RigidBodyHandle,
    green2: RigidBodyHandle,
    blue_team_name: &'static str,
    green_team_name: &'static str,
    blue_team_positive: bool,
    blue_score: usize,
    green_score: usize
}
impl GC {
    pub fn new(
        ball: RigidBodyHandle,
        blue1: RigidBodyHandle,
        blue2: RigidBodyHandle,
        green1: RigidBodyHandle,
        green2: RigidBodyHandle,
        blue_team_name: &'static str,
        green_team_name: &'static str,
        blue_team_positive: bool
    ) -> Self {
        Self {
            state: GCState::Nothing,
            ball,
            blue1,
            blue2,
            green1,
            green2,
            blue_team_name,
            green_team_name,
            blue_team_positive,
            blue_score: 0,
            green_score: 0
        }
    }
    pub fn get_game_state(&self, bodies: &RigidBodySet) -> GameState {
        let ball = bodies[self.ball].translation();
        let blue1 = bodies[self.blue1];
        let blue2 = bodies[self.blue2];
        let green1 = bodies[self.green1];
        let green2 = bodies[self.green2];
        GameState {
            ball: point![ball.x, ball.y],
            markers: Markers {
                blue1: Pose {
                    position: point![blue1.position().translation.x, blue1.position().translation.y],
                    orientation: blue1.position().rotation.angle()
                },
                blue2: Pose {
                    position: point![blue2.position().translation.x, blue2.position().translation.y],
                    orientation: blue2.position().rotation.angle()
                },
                green1: Pose {
                    position: point![green1.position().translation.x, green1.position().translation.y],
                    orientation: green1.position().rotation.angle()
                },
                green2: Pose {
                    position: point![green2.position().translation.x, green2.position().translation.y],
                    orientation: green2.position().rotation.angle()
                }
            },
            referee: Referee {
                teams: RefereeTeams {
                    blue: RefereeTeam {
                        name: self.blue_team_name,
                        x_positive: self.blue_team_positive,
                        score: self.blue_score,
                        robots: RefereeTeamRobots {
                            one: RefereeTeamRobot {
                                penalized: 
                            }
                        }
                    }
                }
            }
        }
    }
}