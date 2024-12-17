mod scheduler;
mod task;
mod workflow;

use scheduler::Scheduler;
use task::Task;
use workflow::Workflow;

#[tokio::main]
async fn main() {
    // Create a new workflow
    let mut workflow = Workflow::new();

    // Define tasks
    let task1 = Task::new(1, "Task 1", "echo Hello from Task 1");
    let task2 = Task::new(2, "Task 2", "echo Hello from Task 2");
    let task3 = Task::new(3, "Task 3", "echo Hello from Task 3");
    let task4 = Task::new(4, "Task 4", "echo Hello from Task 4");

    // Add tasks to the workflow and get their NodeIndex
    let node1 = workflow.add_task(task1);
    let node2 = workflow.add_task(task2);
    let node3 = workflow.add_task(task3);
    let node4 = workflow.add_task(task4);

    // Define dependencies:
    // - Task 3 depends on Task 1 and Task 2
    // - Task 4 depends on Task 3
    workflow.add_dependency(node1, node3);
    workflow.add_dependency(node2, node3);
    workflow.add_dependency(node3, node4);

    // Create a scheduler and run the workflow
    let scheduler = Scheduler::new();
    if let Err(err) = scheduler.run(&workflow).await {
        eprintln!("Error running workflow: {}", err);
    }
}
