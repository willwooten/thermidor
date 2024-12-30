mod scheduler;
mod task;
mod workflow;
mod state;
mod api;
mod sql;
//mod fromdb;

use tracing_subscriber::fmt::init;
use api::run_server;
use workflow::start_workflows;
use sql::{connect_to_database, run_migrations};

/// The main function serves as the entry point of the application.
/// It is an asynchronous function powered by Tokio, allowing concurrent operations.
#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for logging.
    init();

    let database_url = "postgresql://thermidor_user:thermidor_pass@localhost:5432/thermidor";
    let migrations_folder = "./migrations";

    let pool = match connect_to_database(database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Database connection failed: {:?}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = run_migrations(&pool, migrations_folder).await {
        eprintln!("Migration failed: {:?}", e);
        std::process::exit(1);
    }

    // Start workflows.
    let workflows = start_workflows().await;
    run_server(workflows).await;

    // let workflows = match WorkflowManager::load_from_database(&pool).await {
    //     Ok(manager) => Arc::new(Mutex::new(manager)),
    //     Err(err) => {
    //         eprintln!("Failed to initialize workflows: {}", err);
    //         std::process::exit(1);
    //     }
    // };
    // Initialize WorkflowManager from the database
    // let workflows_manager = match WorkflowManager::load_from_database(&pool).await {
    //     Ok(manager) => Arc::new(Mutex::new(manager)),
    //     Err(err) => {
    //         eprintln!("Failed to initialize workflows: {}", err);
    //         std::process::exit(1);
    //     }
    // };

    // // Start workflows using the WorkflowManager
    // start_workflows_from_database(pool.clone(), workflows_manager.clone()).await;

    // Start the server
    // run_server(workflows_manager).await;
}
