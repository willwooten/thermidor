mod scheduler;
mod task;
mod workflow;
mod state;
mod workflow_builder;

use tracing::info;
use tracing_subscriber::fmt::init;
use workflow_builder::WorkflowBuilder;

fn main() {
    // Initialize the tracing subscriber for logging
    init();

    info!("Starting the workflow execution");

    // Create and configure the workflow using WorkflowBuilder
    let mut builder = WorkflowBuilder::new();

    builder
        .add_task(1, "Task 1", "echo Hello from Task 1")
        .add_task(2, "Task 2", "echo Hello from Task 2")
        .add_task(3, "Task 3", "echo Hello from Task 3")
        .add_task(4, "Task 4", "echo Hello from Task 4")
        .add_dependency("Task 1", "Task 3")
        .add_dependency("Task 2", "Task 3")
        .add_dependency("Task 3", "Task 4");

    // Save the workflow to a JSON file
    if let Err(err) = builder.get_workflow().save_to_json("workflow.json") {
        eprintln!("Error saving workflow: {}", err);
    }


    // Export the workflow directly to a PNG image
    builder.export_to_png("workflow.dot", "workflow.png");

    // Run the workflow
    builder.run();
}
