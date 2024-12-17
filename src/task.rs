use tokio::process::Command;
use std::process::Output;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub command: String,
}

impl Task {
    /// Creates a new task with the specified id, name, and command.
    pub fn new(id: usize, name: &str, command: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            command: command.to_string(),
        }
    }

    /// Executes the task asynchronously and prints stdout/stderr.
    pub async fn execute(&self) -> Result<Output, std::io::Error> {
        println!("Executing task {}: {}", self.id, self.name);

        // Split the command into the executable and arguments
        let parts: Vec<&str> = self.command.split_whitespace().collect();
        let (cmd, args) = parts.split_first().unwrap_or((&"", &[]));

        let output = Command::new(cmd)
            .args(args)
            .output()
            .await;

        match &output {
            Ok(result) => {
                if result.status.success() {
                    println!("Task '{}' completed successfully.", self.name);
                    println!("stdout: {}", String::from_utf8_lossy(&result.stdout));
                } else {
                    eprintln!(
                        "Task '{}' failed with exit code: {:?}",
                        self.name,
                        result.status.code()
                    );
                    eprintln!("stderr: {}", String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(err) => eprintln!("Failed to execute task '{}': {}", self.name, err),
        }

        output
    }
}
