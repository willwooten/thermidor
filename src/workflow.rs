use crate::task::Task;
use crate::scheduler::Scheduler;
use petgraph::graph::{DiGraph, NodeIndex};
use sqlx::{PgPool, Row};
use std::fs::File;
use std::io::{self, Write, Read};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use tokio::task;
use tracing::info;

#[derive(Serialize, Deserialize, Clone)]
pub struct Workflow {
    pub graph: DiGraph<Task, ()>,
    pub resumed: bool,
}

impl Workflow {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            resumed: false,
        }
    }

    /// Adds a task to the workflow and returns its NodeIndex.
    pub fn add_task(&mut self, task: Task) -> NodeIndex {
        self.graph.add_node(task)
    }

    /// Adds a dependency between two tasks.
    pub fn add_dependency(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ());
    }

    /// Saves the workflow to a JSON file.
    pub fn save_to_json(&self, filename: &str) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(filename)?;
        file.write_all(json.as_bytes())
    }

    /// Loads the workflow from a JSON file.
    pub fn load_from_json(filename: &str) -> io::Result<Self> {
        let mut file = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let mut workflow: Workflow = serde_json::from_str(&content)?;
        workflow.resumed = true; // Mark the workflow as resumed
        Ok(workflow)
    }

    pub async fn load_from_sql(pool: &PgPool, workflow_id: i64) -> io::Result<Self> {
        // Fetch the workflow data
        let row = sqlx::query("SELECT id, name FROM workflows.workflows WHERE id = $1")
            .bind(workflow_id)
            .fetch_one(pool)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let mut workflow = Workflow::new();

        // Fetch tasks for this workflow
        let task_rows = sqlx::query(
            "SELECT id, task_name, command FROM workflows.tasks WHERE workflow_id = $1"
        )
        .bind(workflow_id)
        .fetch_all(pool)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // Add tasks to the workflow
        for task_row in task_rows {
            let task_id: i64 = task_row.get("id");
            let task_name: String = task_row.get("task_name");
            let task_command: String = task_row.get("command");

            workflow.add_task_dynamically(task_id as usize, &task_name, &task_command);
        }

        // Fetch dependencies for this workflow
        let dependency_rows = sqlx::query(
            "SELECT from_task_idx, to_task_idx FROM workflows.dependencies WHERE workflow_id = $1"
        )
        .bind(workflow_id)
        .fetch_all(pool)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        // Add dependencies to the workflow
        for dependency_row in dependency_rows {
            let from_task_idx: i64 = dependency_row.get("from_task_idx");
            let to_task_idx: i64 = dependency_row.get("to_task_idx");

            workflow.add_dependency_dynamically(from_task_idx as usize, to_task_idx as usize)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        }

        workflow.resumed = true; // Mark the workflow as resumed
        Ok(workflow)
    }
    
    /// Adds a task dynamically and returns the NodeIndex.
    pub fn add_task_dynamically(&mut self, id: usize, name: &str, command: &str) -> NodeIndex {
        let task = Task::new(id, name, command);
        self.add_task(task)
    }

    /// Adds a dependency dynamically.
    pub fn add_dependency_dynamically(&mut self, from: usize, to: usize) -> Result<(), String> {
        let from_node = self.graph.node_indices().find(|&node| self.graph[node].id == from);
        let to_node = self.graph.node_indices().find(|&node| self.graph[node].id == to);

        if let (Some(from_idx), Some(to_idx)) = (from_node, to_node) {
            self.add_dependency(from_idx, to_idx);
            Ok(())
        } else {
            Err("One or both tasks not found".to_string())
        }
    }
    
}

pub struct WorkflowBuilder {
    workflow: Workflow,
    task_indices: HashMap<String, NodeIndex>,
}

impl WorkflowBuilder {
    /// Creates a new WorkflowBuilder.
    pub fn new() -> Self {
        Self {
            workflow: Workflow::new(),
            task_indices: HashMap::new(),
        }
    }

    /// Adds a task to the workflow.
    pub fn add_task(&mut self, id: usize, name: &str, command: &str) -> &mut Self {
        let task = Task::new(id, name, command);
        let node = self.workflow.add_task(task);
        self.task_indices.insert(name.to_string(), node);
        self
    }

    /// Adds a dependency between two tasks.
    pub fn add_dependency(&mut self, from: &str, to: &str) -> &mut Self {
        if let (Some(&from_node), Some(&to_node)) = (self.task_indices.get(from), self.task_indices.get(to)) {
            self.workflow.add_dependency(from_node, to_node);
        } else {
            eprintln!("Error: One or both tasks not found: {} -> {}", from, to);
        }
        self
    }

    /// Provides a cloned copy of the workflow for saving or other operations.
    pub fn get_workflow(&self) -> Workflow {
        self.workflow.clone()
    }
    
}


// Function to create or load workflows
pub async fn schedule_workflow() -> Vec<(Arc<Mutex<Workflow>>, String)> {
    let workflows_data = vec![
        // First example workflow
        ("workflows/workflow1.json", vec![
            (1, "Task 1", "echo Hello from Task 1"),
            (2, "Task 2", "echo Hello from Task 2"),
            (3, "Task 3", "echo Hello from Task 3"),
            (4, "Task 4", "echo Hello from Task 4"),
        ], vec![
            ("Task 1", "Task 3"),
            ("Task 2", "Task 3"),
            ("Task 3", "Task 4"),
        ]),
    ];

    let mut workflows = Vec::new();

    for (workflow_name, tasks, dependencies) in workflows_data {
        let workflow = Arc::new(Mutex::new(
            match Workflow::load_from_json(workflow_name) {
                Ok(wf) => {
                    info!("Loaded workflow from '{}'", workflow_name);
                    wf
                }
                Err(_) => {
                    info!("Creating a new workflow for '{}'", workflow_name);
                    let mut builder = WorkflowBuilder::new();
    
                    // Add tasks
                    for (id, name, command) in tasks {
                        builder.add_task(id, name, command);
                    }
    
                    // Add dependencies
                    for (from, to) in dependencies {
                        builder.add_dependency(from, to);
                    }
    
                    builder.get_workflow()
                }
            },
        ));
    
        workflows.push((workflow, workflow_name.to_string()));
    }

    workflows
}


// Function to start the workflows using the scheduler
async fn _start_workflow(workflows_with_paths: Vec<(Arc<Mutex<Workflow>>, String)>) {
    let scheduler = Scheduler::new();

    for (workflow, save_path) in workflows_with_paths {
        let workflow_clone = Arc::clone(&workflow);
        let scheduler_clone = scheduler.clone();
        let save_path_clone = save_path.clone();

        task::spawn(async move {
            let mut workflow_guard = workflow_clone.lock().await;
            if let Err(err) = scheduler_clone.run(&mut *workflow_guard, &save_path_clone).await {
                eprintln!("Error running workflow '{}': {}", save_path_clone, err);
            }
        });
    }
}

/// Initializes and returns the workflows wrapped in `Arc<Mutex<_>>` for shared access.
pub async fn start_workflows() -> Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>> {
    // Schedule workflows by loading them from configuration or creating new ones.
    let workflows_with_paths = schedule_workflow().await;

    // Start the workflows.
    _start_workflow(workflows_with_paths.clone()).await;

    // Extract workflows and wrap them for shared access.
    let workflows: Vec<_> = workflows_with_paths.into_iter().map(|(wf, _)| wf).collect();
    Arc::new(Mutex::new(workflows))
}
