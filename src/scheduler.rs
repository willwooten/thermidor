use crate::workflow::Workflow;
use petgraph::algo::toposort;
use std::io::Error;
use tracing::{info, error};

pub struct Scheduler;

impl Scheduler {
    /// Creates a new Scheduler.
    pub fn new() -> Self {
        Self
    }

    /// Runs the tasks in the workflow based on their dependencies.
    pub async fn run(&self, workflow: &mut Workflow) -> Result<(), Error> {
        match toposort(&workflow.graph, None) {
            Ok(order) => {
                for node in order {
                    let task = &mut workflow.graph[node]; // Access the Task instance
                    info!("Running task: {}", task.name);
                    if let Err(err) = task.execute().await {
                        error!("Task '{}' failed: {}", task.name, err);
                    }
                    info!("Task '{}' state: {:?}", task.name, task.state);
                }
                Ok(())
            }
            Err(err) => {
                eprintln!("Cycle detected in workflow: {:?}", err);
                Err(Error::new(std::io::ErrorKind::Other, "Cycle detected in workflow"))
            }
        }
    }
}
