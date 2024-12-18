use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Running,
    Success,
    Failure,
}
