mod scheduler;
mod task;
mod workflow;
mod state;
mod workflow_builder;
mod api;
mod workflow_initializer;

use axum::{
    routing::get,
    Router, 
    Extension,
};
use std::sync::Arc;
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

    // Clone the first workflow for use in the API
    let api_workflow = Arc::clone(&workflows[0]);

    // Define the API routes
    let app = Router::new()
        .route("/", get(list_tasks))
        .route("/task/:id", get(get_task))
        .route("/workflow/status", get(get_workflow_status))
        .layer(Extension(api_workflow));

    // Run the HTTP server
    let addr = "0.0.0.0:3000".parse().unwrap();
    info!("Listening on http://{}", addr);
    let server_handle = tokio::spawn(async move {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    // Wait for the server to complete
    let _ = tokio::try_join!(server_handle);
}
