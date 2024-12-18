use crate::workflow::Workflow;
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use std::collections::HashSet;
use std::io::Error;
use tokio::task::JoinHandle;
use tracing::{info, error};
use futures::future::join_all;

pub struct Scheduler;

impl Scheduler {
    /// Creates a new Scheduler.
    pub fn new() -> Self {
        Self
    }

    /// Runs the tasks in the workflow based on their dependencies, with parallel execution.
    pub async fn run(&self, workflow: &mut Workflow) -> Result<(), Error> {
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
                        let task = &mut workflow.graph[node];
                        info!("Scheduling task: {}", task.name);

                        let handle = tokio::spawn({
                            let task = task.clone();
                            async move {
                                let mut task = task.clone(); // Declare task as mutable
                                if let Err(err) = task.execute().await {
                                    error!("Task '{}' failed: {}", task.name, err);
                                }
                                info!("Task '{}' state: {:?}", task.name, task.state);
                                node
                            }
                        });

                        running_tasks.push(handle);
                    }

                    // Collect completed tasks
                    let completed_nodes = join_all(running_tasks.drain(..)).await;
                    for result in completed_nodes {
                        if let Ok(node) = result {
                            completed.insert(node);
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
