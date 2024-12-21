use crate::state::TaskState;
use tokio::process::Command;
use std::process::Output;
use chrono::{DateTime, Utc};
use tokio::time::{sleep, Duration, timeout, Instant};
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
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct NewTask {
    pub id: usize,
    pub name: String,
    pub command: String,
    pub dependencies: Vec<usize>,
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
            timeout_duration: Duration::from_secs(86400), // 24 hours
            start_time: None,
            end_time: None,
        }
    }

    /// Executes the task asynchronously with retry logic and prints stdout/stderr.
    pub async fn execute(&mut self) -> Result<Output, std::io::Error> {
        self.state = TaskState::Running;
        self.start_time = Some(Utc::now()); // Set the task start time

        loop {
            let start_time = Instant::now(); // Track precise execution duration for this attempt

            info!(
                "Executing task {}: {} (Attempt {}/{}) at {:?}",
                self.id,
                self.name,
                self.retry_count + 1,
                self.max_retries + 1,
                start_time // Log the global task start time
            );

            let parts: Vec<&str> = self.command.split_whitespace().collect();
            let (cmd, args) = parts.split_first().unwrap_or((&"", &[]));

            // Execute the command with a timeout
            let output = timeout(self.timeout_duration, Command::new(cmd).args(args).output()).await;

            let end_time = Instant::now(); // Measure end time for this attempt
            let duration = end_time - start_time;

            match output {
                Ok(Ok(result)) => {
                    if result.status.success() {
                        self.state = TaskState::Success;
                        self.end_time = Some(Utc::now()); // Set the task end time
                        info!(
                            "Task '{}' completed successfully in {} seconds.",
                            self.name,
                            duration.as_secs()
                        );
                        info!("stdout: {}", String::from_utf8_lossy(&result.stdout));
                        return Ok(result);
                    } else {
                        self.state = TaskState::Failure;
                        error!(
                            "Task '{}' failed with exit code: {:?} in {} seconds.",
                            self.name,
                            result.status.code(),
                            duration.as_secs()
                        );
                    }
                }
                Ok(Err(err)) => {
                    self.state = TaskState::Failure;
                    error!(
                        "Failed to execute task '{}': {} in {} seconds.",
                        self.name,
                        err,
                        duration.as_secs()
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
                self.end_time = Some(Utc::now()); // Set the task end time on max retry failure
                error!("Task '{}' failed after {} attempts.", self.name, self.retry_count);
                break;
            }

            let retry_delay = Duration::from_secs(2u64.pow(self.retry_count as u32));
            info!("Retrying task '{}' in {:?} seconds...", self.name, retry_delay);
            sleep(retry_delay).await;
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Task '{}' failed after {} retries", self.name, self.retry_count),
        ))
    }
}