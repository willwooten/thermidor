// use crate::workflow::Workflow;
// use crate::workflow_builder::WorkflowBuilder;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use tracing::info;

// pub fn initialize_workflow(save_path: &str) -> Arc<Mutex<Workflow>> {
//     Arc::new(Mutex::new(
//         match Workflow::load_from_json(save_path) {
//             Ok(wf) => {
//                 info!("Loaded workflow from '{}'", save_path);
//                 wf
//             }
//             Err(_) => {
//                 info!("Creating a new workflow.");
//                 let mut builder = WorkflowBuilder::new();

//                 builder
//                     .add_task(1, "Task 1", "echo Hello from Task 1")
//                     .add_task(2, "Task 2", "echo Hello from Task 2")
//                     .add_task(3, "Task 3", "echo Hello from Task 3")
//                     .add_task(4, "Task 4", "echo Hello from Task 4")
//                     .add_task(5, "Long Task", "sleep 30") // New task that sleeps for 30 seconds
//                     .add_dependency("Task 1", "Task 3")
//                     .add_dependency("Task 2", "Task 3")
//                     .add_dependency("Task 3", "Task 4")
//                     .add_dependency("Task 4", "Long Task"); // Dependency for testing

//                 builder.get_workflow()
//             }
//         },
//     ))
// }
use crate::workflow::Workflow;
use crate::workflow_builder::WorkflowBuilder;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub fn initialize_workflow<F>(save_path: &str, setup_fn: F) -> Arc<Mutex<Workflow>>
where
    F: FnOnce(&mut WorkflowBuilder),
{
    Arc::new(Mutex::new(
        match Workflow::load_from_json(save_path) {
            Ok(wf) => {
                info!("Loaded workflow from '{}'", save_path);
                wf
            }
            Err(_) => {
                info!("Creating a new workflow.");
                let mut builder = WorkflowBuilder::new();

                // Call the user-provided setup function to define tasks and dependencies
                setup_fn(&mut builder);

                builder.get_workflow()
            }
        },
    ))
}
