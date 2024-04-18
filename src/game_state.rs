#![cfg_attr(debug_assertions, allow(dead_code))] // TODO remove that later
use rapier2d::prelude::*;

// TODO: Not finished

/* Sample game data:
{
    "markers": {
        "green1": { "position": [-0.5, 0.5], "orientation": 0.0 },
        "green2": { "position": [-0.5, -0.5], "orientation": 0.0 },
        "blue1": { "position": [0.5, 0.5], "orientation": 0.0 },
        "blue2": { "position": [0.5, -0.5], "orientation": 0.0 }
    },
    "ball": [0.0, 0.0],
    "referee": {
        "game_is_running": false,
        "game_paused": true,
        "halftime_is_running": false,
        "timer": 0,
        "game_state_msg": "Game is ready to start",
        "teams": {
            "green": {
                "name": "",
                "score": 0,
                "x_positive": true,
                "robots": {
                    "1": {
                        "penalized": false,
                        "penalized_remaining": null,
                        "penalized_reason": null,
                        "preempted": false,
                        "preemption_reasons": []
                    },
                    "2": {
                        "penalized": false,
                        "penalized_remaining": null,
                        "penalized_reason": null,
                        "preempted": false,
                        "preemption_reasons": []
                    }
                }
            },
            "blue": {
                "name": "",
                "score": 0,
                "x_positive": false,
                "robots": {
                    "1": {
                        "penalized": false,
                        "penalized_remaining": null,
                        "penalized_reason": null,
                        "preempted": false,
                        "preemption_reasons": []
                    },
                    "2": {
                        "penalized": false,
                        "penalized_remaining": null,
                        "penalized_reason": null,
                        "preempted": false,
                        "preemption_reasons": []
                    }
                }
            }
        },
        "referee_history_sliced": [0, -9263, "neutral", "Sideline crossed"]
    },
    "leds": {
        "green1": [0, 50, 0],
        "green2": [0, 50, 0],
        "blue1": [0, 0, 50],
        "blue2": [0, 0, 50]
    },
    "simulated": true
}
*/

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pose {
    pub position: Point<f32>,
    pub orientation: f32,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Markers {
    pub green1: Pose,
    pub green2: Pose,
    pub blue1: Pose,
    pub blue2: Pose,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RefereeTeamRobot {
    pub penalized: bool,
    pub penalized_remaining: Option<usize>,
    pub penalized_reson: Option<String>,
    pub preempted: bool,
    pub preemption_reasons: Vec<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RefereeTeamRobots {
    pub one: RefereeTeamRobot,
    pub two: RefereeTeamRobot,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RefereeTeam {
    pub name: String,
    pub score: usize,
    pub x_positive: bool,
    pub robots: RefereeTeamRobots,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RefereeTeams {
    pub green: RefereeTeam,
    pub blue: RefereeTeam,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Referee {
    pub game_is_running: bool,
    pub game_paused: bool,
    pub halftime_is_running: bool,
    pub timer: usize,
    pub game_state_msg: String,
    pub teams: RefereeTeams,
    //pub referee_history_sliced: // TODO: Fill this to be complient with the official rsk game_controller
}

/// Representation of the game given to the client
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameState {
    pub ball: Option<Point<f32>>,
    pub markers: Markers,
    pub referee: Referee,
}
