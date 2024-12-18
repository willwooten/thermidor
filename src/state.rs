#[derive(Debug, Clone)]
pub enum TaskState {
    Pending,
    Running,
    Success,
    Failure,
}
