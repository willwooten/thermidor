use crate::state::TaskState;
use tokio::process::Command;
use std::process::Output;
use chrono::Utc;
use tracing::{info, error};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub command: String,
    pub state: TaskState,
}

impl Task {
    /// Creates a new task with the specified id, name, and command.
    pub fn new(id: usize, name: &str, command: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            command: command.to_string(),
            state: TaskState::Pending,
        }
    }

    /// Executes the task asynchronously and logs stdout/stderr, with execution time tracking.
    pub async fn execute(&mut self) -> Result<Output, std::io::Error> {
        self.state = TaskState::Running;
        let start_time = Utc::now();
        info!("Executing task {}: {} at {}", self.id, self.name, start_time);

        let parts: Vec<&str> = self.command.split_whitespace().collect();
        let (cmd, args) = parts.split_first().unwrap_or((&"", &[]));

        let output = Command::new(cmd).args(args).output().await;

        let end_time = Utc::now();
        let duration = end_time - start_time;

        match &output {
            Ok(result) => {
                if result.status.success() {
                    self.state = TaskState::Success;
                    info!(
                        "Task '{}' completed successfully in {} seconds.",
                        self.name,
                        duration.num_seconds()
                    );
                    info!("stdout: {}", String::from_utf8_lossy(&result.stdout));
                } else {
                    self.state = TaskState::Failure;
                    error!(
                        "Task '{}' failed with exit code: {:?} in {} seconds.",
                        self.name,
                        result.status.code(),
                        duration.num_seconds()
                    );
                    error!("stderr: {}", String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(err) => {
                self.state = TaskState::Failure;
                error!(
                    "Failed to execute task '{}': {} in {} seconds.",
                    self.name,
                    err,
                    duration.num_seconds()
                );
            }
        }

        output
    }
}
