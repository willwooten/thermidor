use crate::workflow::Workflow;
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use std::collections::HashSet;
use std::io::Error;
use tokio::task::JoinHandle;
use tracing::{info, error};
use futures::future::join_all;

#[derive(Clone, Copy)]
pub struct Scheduler;

impl Scheduler {
    /// Creates a new Scheduler.
    pub fn new() -> Self {
        Self
    }

    /// Runs the tasks in the workflow based on their dependencies, with parallel execution and state persistence.
    pub async fn run(&self, workflow: &mut Workflow, save_path: &str) -> Result<(), Error> {
        match toposort(&workflow.graph, None) {
            Ok(order) => {
                let mut running_tasks: Vec<JoinHandle<NodeIndex>> = Vec::new();
                let mut completed = HashSet::new();

                for node in order {
                    // Wait for tasks whose dependencies haven't completed
                    running_tasks.retain(|handle| !handle.is_finished());

                    // Check if all dependencies of the current task have completed
                    let all_deps_completed = workflow
                        .graph
                        .neighbors_directed(node, petgraph::Incoming)
                        .all(|dep| completed.contains(&dep));

                    if all_deps_completed {
                        let task = workflow.graph[node].clone();
                        info!("Scheduling task: {}", task.name);

                        let handle = tokio::spawn({
                            let mut task_clone = task.clone();
                            async move {
                                if let Err(err) = task_clone.execute().await {
                                    error!("Task '{}' failed: {}", task_clone.name, err);
                                } else {
                                    info!("Task '{}' completed successfully.", task_clone.name);
                                }
                                node
                            }
                        });

                        running_tasks.push(handle);
                    }

                    // Collect completed tasks
                    let completed_nodes = join_all(running_tasks.drain(..)).await;
                    for result in completed_nodes {
                        if let Ok(node) = result {
                            // Update the graph with the modified task state
                            workflow.graph[node] = workflow.graph[node].clone();
                            completed.insert(node);

                            // Save the workflow state after each task execution
                            if let Err(err) = workflow.save_to_json(&save_path) {
                                error!("Failed to save workflow state: {}", err);
                            }
                        }
                    }
                }

                Ok(())
            }
            Err(err) => {
                error!("Cycle detected in workflow: {:?}", err);
                Err(Error::new(std::io::ErrorKind::Other, "Cycle detected in workflow"))
            }
        }
    }
}
