use crate::workflow::Workflow;
use crate::workflow_builder::WorkflowBuilder;
use crate::scheduler::Scheduler;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub async fn initialize_workflows() -> Vec<Arc<Mutex<Workflow>>> {
    let workflows_data = vec![
        // First example workflow
        ("workflow1.json", vec![
            (1, "Task 1", "echo Hello from Task 1"),
            (2, "Task 2", "echo Hello from Task 2"),
            (3, "Task 3", "echo Hello from Task 3"),
            (4, "Task 4", "echo Hello from Task 4"),
            (5, "Long Task", "sleep 30"),
        ], vec![
            ("Task 1", "Task 3"),
            ("Task 2", "Task 3"),
            ("Task 3", "Task 4"),
            ("Task 4", "Long Task"),
        ]),

        // Second example workflow
        // ("workflow2.json", vec![
        //     (1, "Download Data", "curl -O https://example.com/data.csv"),
        //     (2, "Process Data", "python process_data.py"),
        //     (3, "Generate Report", "python generate_report.py"),
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
