mod scheduler;
mod task;
mod workflow;
mod state;
mod workflow_builder;
mod api;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::fmt::init;
use api::run_server;
use workflow::initialize_workflows;

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for logging
    init();

    // Initialize and run multiple workflows
    let workflows = initialize_workflows().await;
    let workflows = Arc::new(Mutex::new(workflows));

    // Run the HTTP server
    run_server(workflows).await;
}
