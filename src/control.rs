#[cfg(feature = "serde")]
use serde::{ser::SerializeTuple, Serialize};

#[derive(Debug)]
pub enum CtrlRes {
    UnknownError,
    /// (team)
    BadKey(String),
    Preempted(String, u8, String),
    UnknownRobot(String, u8),
    UnknownCommand,
    Ok
}
#[cfg(feature = "serde")]
impl Serialize for CtrlRes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        match self {
            &CtrlRes::UnknownError => {
                // [False, "Unknown error"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&false)?;
                tup.serialize_element("Unknown error")?;
                tup.end()
            },
            &CtrlRes::BadKey(ref team) => {
                // [False, "Bad key for team {team}"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&false)?;
                tup.serialize_element(&format!("Bad key for team {}", team))?;
                tup.end()
            },
            &CtrlRes::Preempted(ref team, robot_number, ref reason) => {
                // [2, "Robot {number} of team {team} is preempted: {reasons}"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&2)?;
                tup.serialize_element(&format!("Robot {} of team {} is preempted: {}", robot_number, team, reason))?;
                tup.end()
            },
            &CtrlRes::UnknownRobot(ref team, robot_number) => {
                // [False, "Unknown robot: {marker}"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&false)?;
                tup.serialize_element(&format!("Unknown robot: {}{}", team, robot_number))?;
                tup.end()
            },
            
            &CtrlRes::UnknownCommand => {
                // [2, "Unknown command"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&2)?;
                tup.serialize_element("Unknown command")?;
                tup.end()
            },
            &CtrlRes::Ok => {
                // [True, "ok"]
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(&true)?;
                tup.serialize_element("ok")?;
                tup.end()
            }
        }
    }
}