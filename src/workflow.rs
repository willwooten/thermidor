use crate::task::Task;
use crate::scheduler::Scheduler;
use petgraph::graph::{DiGraph, NodeIndex};
use std::fs::File;
use std::io::{self, Write, Read};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use tracing::info;


#[derive(Serialize, Deserialize, Clone)]
pub struct Workflow {
    pub graph: DiGraph<Task, ()>,
}

impl Workflow {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
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

    /// Exports the workflow to a DOT file.
    // pub fn export_to_dot(&self, filename: &str) -> io::Result<()> {
    //     let dot = format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]));
    //     let mut file = File::create(filename)?;
    //     file.write_all(dot.as_bytes())
    // }

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
        let workflow = serde_json::from_str(&content)?;
        Ok(workflow)
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

    /// Runs the workflow using the scheduler.
    // pub fn run(&mut self, save_path: &str) {
    //     let scheduler = Scheduler::new();
    //     let rt = Runtime::new().unwrap();

    //     rt.block_on(async {
    //         if let Err(err) = scheduler.run(&mut self.workflow, save_path).await {
    //             eprintln!("Error running workflow: {}", err);
    //         }
    //     });
    // }

    /// Exports the workflow to a PNG image by generating a DOT file and running the dot command.
    // pub fn export_to_png(&self, dot_filename: &str, png_filename: &str) {
    //     // Export the workflow to a DOT file
    //     if let Err(err) = self.workflow.export_to_dot(dot_filename) {
    //         eprintln!("Error exporting workflow to DOT file: {}", err);
    //         return;
    //     }

    //     // Generate the PNG using the dot command
    //     let output = Command::new("dot")
    //         .args(&["-Tpng", dot_filename, "-o", png_filename])
    //         .output();

    //     match output {
    //         Ok(result) => {
    //             if result.status.success() {
    //                 println!("Workflow exported to {}", png_filename);
    //             } else {
    //                 eprintln!(
    //                     "Error running dot command: {}",
    //                     String::from_utf8_lossy(&result.stderr)
    //                 );
    //             }
    //         }
    //         Err(err) => {
    //             eprintln!("Failed to execute dot command: {}", err);
    //         }
    //     }
    // }

    /// Provides a cloned copy of the workflow for saving or other operations.
    pub fn get_workflow(&self) -> Workflow {
        self.workflow.clone()
    }

    // Creates a WorkflowBuilder from an existing workflow.
    // pub fn from_workflow(workflow: Workflow) -> Self {
    //     Self {
    //         workflow,
    //         task_indices: HashMap::new(), // You may need to reconstruct this map
    //     }
    // }
    
}

pub async fn initialize_workflows() -> Vec<Arc<Mutex<Workflow>>> {
    let workflows_data = vec![
        // First example workflow
        ("workflow1.json", vec![
            (1, "Task 1", "echo Hello from Task 1"),
            (2, "Task 2", "echo Hello from Task 2"),
            (3, "Task 3", "echo Hello from Task 3"),
            (4, "Task 4", "echo Hello from Task 4"),
        ], vec![
            ("Task 1", "Task 3"),
            ("Task 2", "Task 3"),
            ("Task 3", "Task 4"),
        ]),

        // Second example workflow
        // ("workflow2.json", vec![
        //     (1, "Download Data", "echo Hello from Task 5"),
        //     (2, "Process Data", "echo Hello from Task 6"),
        //     (3, "Generate Report", "echo Hello from Task 7"),
        // ], vec![
        //     ("Download Data", "Process Data"),
        //     ("Process Data", "Generate Report"),
        // ]),

        // Third example workflow
        // ("workflow3.json", vec![
        //     (1, "Compile Code", "cargo build"),
        //     (2, "Run Tests", "cargo test"),
        //     (3, "Deploy", "echo Deploying application"),
        // ], vec![
        //     ("Compile Code", "Run Tests"),
        //     ("Run Tests", "Deploy"),
        // ]),
    ];

    let mut workflows = Vec::new();
    let scheduler = Scheduler::new();

    for (save_path, tasks, dependencies) in workflows_data {
        let workflow = Arc::new(Mutex::new(
            match Workflow::load_from_json(save_path) {
                Ok(wf) => {
                    info!("Loaded workflow from '{}'", save_path);
                    wf
                }
                Err(_) => {
                    info!("Creating a new workflow for '{}'", save_path);
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

        // Run the workflow using the scheduler concurrently
        let workflow_clone = Arc::clone(&workflow);
        let save_path_clone = save_path.to_string();
        let scheduler_clone = scheduler.clone();

        tokio::spawn(async move {
            let mut workflow_guard = workflow_clone.lock().await;
            if let Err(err) = scheduler_clone.run(&mut *workflow_guard, &save_path_clone).await {
                eprintln!("Error running workflow '{}': {}", save_path_clone, err);
            }
        });

        workflows.push(workflow);
    }

    workflows
}
