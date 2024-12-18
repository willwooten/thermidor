use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskState {
    /// The task is waiting to be executed.
    Pending,

    /// The task is currently being executed.
    Running,

    /// The task completed successfully.
    Success,

    /// The task encountered an error and failed.
    Failure,

    /// The task was skipped because its dependencies were not met.
    Skipped,
}
