use crate::task::Task;
use crate::workflow::Workflow;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

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
