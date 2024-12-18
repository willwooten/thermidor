mod scheduler;
mod task;
mod workflow;
mod state;
mod api;

use std::sync::Arc;          // For thread-safe reference counting and shared ownership
use tokio::sync::Mutex;      // For asynchronous, thread-safe mutable access to data
use tracing_subscriber::fmt::init;
use api::run_server;
use workflow::initialize_workflows;

/// The main function serves as the entry point of the application.
/// It is an asynchronous function powered by Tokio, allowing concurrent operations.
#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for logging.
    init();

    // Initialize workflows by loading them from configuration or creating new ones.
    // This function returns a vector of workflows that are ready to be executed.
    let workflows = initialize_workflows().await;

    // Wrap the workflows in an `Arc` (Atomic Reference Counted pointer) and a `Mutex` (mutual exclusion).
    // - `Arc`: Ensures that the workflows can be shared safely across multiple threads.
    // - `Mutex`: Allows asynchronous, thread-safe mutable access to the workflows.
    let workflows = Arc::new(Mutex::new(workflows));

    // Start the HTTP server to expose the API endpoints for interacting with the workflows.
    run_server(workflows).await;
}
