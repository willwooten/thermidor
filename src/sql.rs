use sqlx::{PgPool, Row};
use glob::glob;
use std::error::Error;
use tokio::fs::read_to_string;
use std::path::Path;

// Connect to the database
pub async fn connect_to_database(database_url: &str) -> Result<PgPool, Box<dyn Error>> {
    println!("Connecting to the database...");
    let pool = PgPool::connect(database_url).await?;
    println!("Database connection established.");
    Ok(pool)
}

// Execute a single migration file
async fn execute_migration(pool: &PgPool, path: &Path) -> Result<(), Box<dyn Error>> {
    println!("Executing migration: {}", path.display());

    // Read the migration file content
    let sql = read_to_string(path).await?;

    // Split the SQL into individual statements
    let statements: Vec<&str> = sql.split(';')
        .filter(|stmt| !stmt.trim().is_empty())
        .collect();

    // Execute each statement separately
    for statement in statements {
        sqlx::query(statement).execute(pool).await?;
    }

    println!("Migration {} executed successfully.", path.display());
    Ok(())
}

// Run all migration files in the specified folder
async fn run_all_migrations(pool: &PgPool, migrations_folder: &str) -> Result<(), Box<dyn Error>> {
    let pattern = format!("{}/*.sql", migrations_folder);
    let mut entries: Vec<_> = glob(&pattern)?.filter_map(Result::ok).collect();

    // Sort files based on their names (lexicographical order ensures numeric sorting like 01_, 02_)
    entries.sort();

    for path in entries {
        if let Err(e) = execute_migration(pool, &path).await {
            eprintln!("Failed to execute migration {}: {:?}", path.display(), e);
            return Err(e);
        }
    }
    Ok(())
}

pub async fn run_migrations(pool: &PgPool, migrations_folder: &str) -> Result<(), Box<dyn Error>> {
    run_all_migrations(&pool, migrations_folder).await?;
    println!("Database migrations completed successfully.");
    Ok(())
}

// Insert a new workflow
pub async fn insert_workflow(pool: &PgPool, name: &str) -> Result<i64, Box<dyn Error>> {
    let row = sqlx::query("INSERT INTO workflows.workflows (name, status) VALUES ($1, $2) RETURNING id")
        .bind(name)
        .bind("Stopped")
        .fetch_one(pool)
        .await?;

    Ok(row.get("id"))
}

// Insert a new task
pub async fn insert_task(
    pool: &PgPool,
    workflow_id: i64,
    task_name: &str,
    command: &str,
) -> Result<i64, Box<dyn Error>> {
    let row = sqlx::query(
        "INSERT INTO workflows.tasks (workflow_id, task_name, command) VALUES ($1, $2, $3) RETURNING id"
    )
    .bind(workflow_id)
    .bind(task_name)
    .bind(command)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

// Update task status
pub async fn update_task_status(
    pool: &PgPool,
    task_id: i64,
    status: &str
) -> Result<(), Box<dyn Error>> {
    sqlx::query("UPDATE workflows.task_status SET status = $1, updated_at = NOW() WHERE task_id = $2")
        .bind(status)
        .bind(task_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_workflow_id(pool: &PgPool, workflow_name: String) -> Result<(i64), Box<dyn Error>> {
    let row = sqlx::query("SELECT id FROM workflows.workflows WHERE name = $1")
        .bind(workflow_name)
        .fetch_one(pool)
        .await?;

    let id: i64 = row.get("id");

    Ok(id)
}

// Get tasks by workflow ID
pub async fn get_tasks(pool: &PgPool, workflow_id: i64) -> Result<Vec<(i64, String, String, String)>, Box<dyn Error>> {
    let rows = sqlx::query("SELECT id, task_name, command, state FROM workflows.tasks WHERE workflow_id = $1")
        .bind(workflow_id)
        .fetch_all(pool)
        .await?;

    let tasks = rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.get("id");
            let name: String = row.get("task_name");
            let command: String = row.get("command");
            let state: String = row.get("state");
            (id, name, command, state)
        })
        .collect();

    Ok(tasks)
}

// Get dependencies for a workflow
pub async fn get_dependencies(
    pool: &PgPool,
    workflow_id: i64
) -> Result<Vec<(i64, i64, i64)>, Box<dyn Error>> {
    let rows = sqlx::query(
        "SELECT id, from_task_id, to_task_id FROM workflows.dependencies WHERE workflow_id = $1"
    )
    .bind(workflow_id)
    .fetch_all(pool)
    .await?;

    let dependencies = rows
        .into_iter()
        .map(|row| {
            let id: i64 = row.get("id");
            let from_task_id: i64 = row.get("from_task_id");
            let to_task_id: i64 = row.get("to_task_id");
            (id, from_task_id, to_task_id)
        })
        .collect();

    Ok(dependencies)
}