mod scheduler;
mod task;
mod workflow;
mod state;
mod workflow_builder;
mod api;
mod workflow_initializer;

use axum::{routing::get, Router, Extension};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::fmt::init;
use api::{list_tasks, get_task, get_workflow_status};
use workflow_initializer::initialize_workflows;

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for logging
    init();

    // Initialize and run multiple workflows
    let workflows = initialize_workflows().await;
    let workflows = Arc::new(Mutex::new(workflows));

    // Define the API routes
    let app = Router::new()
        .route("/workflows", get(list_tasks))
        .route("/workflow/:workflow_id/task/:id", get(get_task))
        .route("/workflow/:workflow_id/status", get(get_workflow_status))
        .layer(Extension(workflows));

    // Run the HTTP server
    let addr = "0.0.0.0:3000".parse().unwrap();
    info!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
