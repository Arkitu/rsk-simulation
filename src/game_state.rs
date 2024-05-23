use std::{f64::consts::PI, ops::{Add, Sub}};

// TODO remove that later
use rapier2d_f64::prelude::*;

use crate::constants::real::*;

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
    pub position: Point<f64>,
    pub orientation: f64,
}
impl Add for &Pose {
    type Output = Pose;
    fn add(self, rhs: Self) -> Self::Output {
        Pose {
            position: Point::<f64>::new(self.position.x + rhs.position.x, self.position.y + rhs.position.y) ,
            orientation: self.orientation + rhs.orientation
        }
    }
}
impl Sub for &Pose {
    type Output = Pose;
    fn sub(self, rhs: Self) -> Self::Output {
        Pose {
            position: Point::<f64>::new(self.position.x - rhs.position.x, self.position.y - rhs.position.y) ,
            orientation: self.orientation - rhs.orientation
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Markers {
    pub blue1: Pose,
    pub blue2: Pose,
    pub green1: Pose,
    pub green2: Pose,
}
impl Default for Markers {
    fn default() -> Self {
        Self {
            blue1: Pose { position: DEFAULT_ROBOTS_POS[0], orientation: 0. },
            blue2: Pose { position: DEFAULT_ROBOTS_POS[1], orientation: 0. },
            green1: Pose { position: DEFAULT_ROBOTS_POS[2], orientation: PI },
            green2: Pose { position: DEFAULT_ROBOTS_POS[3], orientation: PI }
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RefereeTeamRobot {
    pub penalized: bool,
    pub penalized_remaining: Option<usize>,
    pub penalized_reason: Option<String>,
    pub preempted: bool,
    pub preemption_reasons: Vec<String>,
}
impl Default for RefereeTeamRobot {
    fn default() -> Self {
        Self {
            penalized: false,
            penalized_remaining: None,
            penalized_reason: None,
            preempted: false,
            preemption_reasons: Vec::new()
        }
    }
}

#[derive(Clone, Debug, Default)]
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
impl Default for RefereeTeam {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            score: 0,
            x_positive: true,
            robots: RefereeTeamRobots::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
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
impl Default for Referee {
    fn default() -> Self {
        Self {
            game_is_running: false,
            game_paused: false,
            halftime_is_running: false,
            timer: 0,
            game_state_msg: "Game is ready to start".to_string(),
            teams: RefereeTeams::default()
        }
    }
}

/// Representation of the game given to the client
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameState {
    pub ball: Option<Point<f64>>,
    pub markers: Markers,
    pub referee: Referee,
}
impl Default for GameState {
    fn default() -> Self {
        Self {
            ball: None,
            markers: Markers::default(),
            referee: Referee::default()
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_gui", derive(bevy::prelude::Component))]
/// `robot as usize` can be used to index arrays of things related to robots. For example you can get the default position of blue2 with `DEFAULT_ROBOTS_POS[Robot::Blue2 as usize]`
pub enum Robot {
    Blue1 = 0,
    Blue2 = 1,
    Green1 = 2,
    Green2 = 3,
}
impl Robot {
    pub const fn all() -> [Self; 4] {
        [Self::Blue1, Self::Blue2, Self::Green1, Self::Green2]
    }
}

#[derive(Default)]
pub struct RobotTasks {
    /// (reason, start)
    pub penalty: Option<(&'static str, usize)>,
    /// (x, y, rotation)
    pub control: Option<(f32, f32, f32)>,
    /// strength
    pub kick: Option<f32>
}
impl RobotTasks {
    pub fn preemption_reason(&self, robot: Robot) -> Option<&'static str> {
        self.penalty.map(|(_, _)| match robot {
            Robot::Blue1 => "penalty-blue1",
            Robot::Blue2 => "penalty-blue2",
            Robot::Green1 => "penalty-green1",
            Robot::Green2 => "penalty-green2"
        })
    }
}