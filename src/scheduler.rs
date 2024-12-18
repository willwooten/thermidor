use crate::workflow::Workflow;
use crate::state::TaskState;
use crate::task::Task;
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use std::collections::HashSet;
use std::io::{Error, Write};
use tokio::task::JoinHandle;
use tracing::{info, error};
use futures::future::join_all;
use std::path::Path;
use std::fs;

#[derive(Clone, Copy)]
pub struct Scheduler;

impl Scheduler {
    /// Creates a new Scheduler.
    pub fn new() -> Self {
        Self
    }

    /// Executes a single task asynchronously.
    async fn execute_task(node: NodeIndex, mut task: Task) -> Result<(NodeIndex, TaskState), String> {
        info!("Scheduling task: {}", task.name);
    
        let state = if let Err(err) = task.execute().await {
            error!("Task '{}' failed: {}", task.name, err);
            TaskState::Failure
        } else {
            info!("Task '{}' completed successfully.", task.name);
            TaskState::Success
        };
    
        // Force flush of the log buffer
        std::io::stdout().flush().unwrap();
    
        Ok((node, state))
    }
    

    /// Runs the tasks in the workflow based on their dependencies, with parallel execution and state persistence.
    pub async fn run(&self, workflow: &mut Workflow, save_path: &str) -> Result<(), Error> {
        // Ensure the directory exists
        if let Some(parent_dir) = Path::new(save_path).parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir)?;
            }
        }
    
        // Reset all tasks to Pending state at the start
        for node in workflow.graph.node_indices() {
            workflow.graph[node].state = TaskState::Pending;
        }
    
        let mut completed = HashSet::new();
    
        loop {
            let mut running_tasks: Vec<JoinHandle<Result<(NodeIndex, TaskState), String>>> = Vec::new();
            let mut progress_made = false;
    
            match toposort(&workflow.graph, None) {
                Ok(order) => {
                    for node in order {
                        running_tasks.retain(|handle| !handle.is_finished());
    
                        if workflow.graph[node].state == TaskState::Pending || workflow.graph[node].state == TaskState::Skipped {
                            let all_deps_completed = workflow
                                .graph
                                .neighbors_directed(node, petgraph::Incoming)
                                .all(|dep| completed.contains(&dep));
    
                            if all_deps_completed {
                                let task = workflow.graph[node].clone();
                                let handle = tokio::spawn(Self::execute_task(node, task));
                                running_tasks.push(handle);
                                progress_made = true;
                            } else {
                                info!("Skipping task: {} due to incomplete dependencies", workflow.graph[node].name);
                                workflow.graph[node].state = TaskState::Skipped;
                            }
                        }
                    }
    
                    // Collect completed tasks
                    let completed_nodes = join_all(running_tasks).await;
                    for result in completed_nodes {
                        match result {
                            Ok(Ok((node, state))) => {
                                workflow.graph[node].state = state;
                                completed.insert(node);
    
                                // Save the workflow state after each task execution
                                if let Err(err) = workflow.save_to_json(&save_path) {
                                    error!("Failed to save workflow state: {}", err);
                                }
                            }
                            Ok(Err(err)) => {
                                error!("Task execution error: {}", err);
                            }
                            Err(join_err) => {
                                error!("Join error: {}", join_err);
                            }
                        }
                    }
    
                    // If no progress was made and there are still skipped tasks, break the loop
                    if !progress_made {
                        break;
                    }
                }
                Err(err) => {
                    error!("Cycle detected in workflow: {:?}", err);
                    return Err(Error::new(std::io::ErrorKind::Other, "Cycle detected in workflow"));
                }
            }
        }
    
        Ok(())
    }    
}
