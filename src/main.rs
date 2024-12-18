mod scheduler;
mod task;
mod workflow;
mod state;
mod workflow_builder;

use tracing::info;
use tracing_subscriber::fmt::init;
use workflow::Workflow;
use scheduler::Scheduler;

fn main() {
    // Initialize the tracing subscriber for logging
    init();

    let save_path = "workflow.json";

    // Load the workflow from a saved state or create a new one
    let mut workflow = match Workflow::load_from_json(save_path) {
        Ok(wf) => {
            info!("Loaded workflow from '{}'", save_path);
            wf
        }
        Err(_) => {
            info!("Creating a new workflow.");
            let mut builder = workflow_builder::WorkflowBuilder::new();

            builder
                .add_task(1, "Task 1", "echo Hello from Task 1")
                .add_task(2, "Task 2", "echo Hello from Task 2")
                .add_task(3, "Task 3", "echo Hello from Task 3")
                .add_task(4, "Task 4", "echo Hello from Task 4")
                .add_dependency("Task 1", "Task 3")
                .add_dependency("Task 2", "Task 3")
                .add_dependency("Task 3", "Task 4");

            builder.get_workflow().clone()
        }
    };

    // Run the workflow and save state after each task
    let scheduler = Scheduler::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        if let Err(err) = scheduler.run(&mut workflow, save_path).await {
            eprintln!("Error running workflow: {}", err);
        }
    });
}
