mod scheduler;
mod task;
mod workflow;
mod state;
mod api;

use tracing_subscriber::fmt::init;
use api::run_server;
use workflow::start_workflows;


/// The main function serves as the entry point of the application.
/// It is an asynchronous function powered by Tokio, allowing concurrent operations.
#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for logging.
    init();

    // Start workflows.
    let workflows = start_workflows().await;

    // Start the HTTP server to expose the API endpoints for interacting with the workflows.
    run_server(workflows).await;
}
