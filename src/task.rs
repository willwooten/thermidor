use crate::state::TaskState;
use tokio::process::Command;
use std::process::Output;
use chrono::Utc;
use tokio::time::{sleep, Duration, timeout};
use tracing::{info, error};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub command: String,
    pub state: TaskState,
    pub max_retries: usize,
    pub retry_count: usize,
    pub timeout_duration: Duration,
}

impl Task {
    /// Creates a new task with the specified id, name, command, and max retries.
    pub fn new(id: usize, name: &str, command: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            command: command.to_string(),
            state: TaskState::Pending,
            max_retries: 5,
            retry_count: 0,
            timeout_duration: Duration::from_secs(86400) // 24 hours
        }
    }

    /// Executes the task asynchronously with retry logic and prints stdout/stderr.
    pub async fn execute(&mut self) -> Result<Output, std::io::Error> {
        self.state = TaskState::Running;

        loop {
            let start_time = Utc::now();

            info!(
                "Executing task {}: {} (Attempt {}/{}) at {}",
                self.id,
                self.name,
                self.retry_count + 1,
                self.max_retries + 1,
                start_time
            );

            let parts: Vec<&str> = self.command.split_whitespace().collect();
            let (cmd, args) = parts.split_first().unwrap_or((&"", &[]));

            // Execute the command with a timeout
            let output = timeout(self.timeout_duration, Command::new(cmd).args(args).output()).await;

            let end_time = Utc::now();
            let duration = end_time - start_time;

            match output {
                Ok(Ok(result)) => {
                    if result.status.success() {
                        self.state = TaskState::Success;
                        info!(
                            "Task '{}' completed successfully in {} seconds.",
                            self.name,
                            duration.num_seconds()
                        );
                        info!("stdout: {}", String::from_utf8_lossy(&result.stdout));
                        return Ok(result);
                    } else {
                        self.state = TaskState::Failure;
                        error!(
                            "Task '{}' failed with exit code: {:?} in {} seconds.",
                            self.name,
                            result.status.code(),
                            duration.num_seconds()
                        );
                    }
                }
                Ok(Err(err)) => {
                    self.state = TaskState::Failure;
                    error!(
                        "Failed to execute task '{}': {} in {} seconds.",
                        self.name,
                        err,
                        duration.num_seconds()
                    );
                }
                Err(_) => {
                    self.state = TaskState::Failure;
                    error!(
                        "Task '{}' timed out after {} seconds.",
                        self.name,
                        self.timeout_duration.as_secs()
                    );
                }
            }

            self.retry_count += 1;

            if self.retry_count > self.max_retries {
                error!("Task '{}' failed after {} attempts.", self.name, self.retry_count);
                break;
            }

            let retry_delay = Duration::from_secs(2u64.pow(self.retry_count as u32));
            info!("Retrying task '{}' in {:?} seconds...", self.name, retry_delay);
            sleep(retry_delay).await;            
        }

        Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Task '{}' failed after {} retries", self.name, self.retry_count)))
    }
}