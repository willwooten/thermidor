use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
pub enum TaskState {
    Pending,
    Running,
    Success,
    Failure,
    Skipped,
    Stopped
}

impl FromStr for TaskState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(TaskState::Pending),
            "Running" => Ok(TaskState::Running),
            "Success" => Ok(TaskState::Success),
            "Failure" => Ok(TaskState::Failure),
            "Skipped" => Ok(TaskState::Skipped),
            "Stopped" => Ok(TaskState::Stopped),
            _ => Err(format!("Invalid task state: {}", s)),
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = match self {
            TaskState::Pending => "Pending",
            TaskState::Running => "Running",
            TaskState::Success => "Success",
            TaskState::Failure => "Failure",
            TaskState::Skipped => "Skipped",
            TaskState::Stopped => "Stopped",
        };
        write!(f, "{}", state)
    }
}