use crate::workflow::Workflow;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json, Extension};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

// List all tasks with their statuses
pub async fn list_tasks(
    Extension(workflow): Extension<Arc<Mutex<Workflow>>>,
) -> impl IntoResponse {
    let workflow = workflow.lock().await;
    let tasks: Vec<_> = workflow
        .graph
        .node_weights()
        .map(|task| {
            json!({
                "id": task.id,
                "name": task.name,
                "state": format!("{:?}", task.state),
            })
        })
        .collect();

    Json(tasks)
}

// Get details of a specific task by ID
pub async fn get_task(
    Path(id): Path<usize>,
    Extension(workflow): Extension<Arc<Mutex<Workflow>>>,
) -> impl IntoResponse {
    let workflow = workflow.lock().await;

    if let Some(task) = workflow.graph.node_weights().find(|task| task.id == id) {
        Json(json!({
            "id": task.id,
            "name": task.name,
            "command": task.command,
            "state": format!("{:?}", task.state),
        }))
        .into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Task not found" }))).into_response()
    }
}

// Get the status of the entire workflow
pub async fn get_workflow_status(
    Extension(workflow): Extension<Arc<Mutex<Workflow>>>,
) -> impl IntoResponse {
    let workflow = workflow.lock().await;
    let states: Vec<_> = workflow
        .graph
        .node_weights()
        .map(|task| format!("{:?}", task.state))
        .collect();

    let status = if states.iter().all(|s| s == "Success") {
        "Completed"
    } else if states.contains(&"Failure".to_string()) {
        "Failed"
    } else {
        "In Progress"
    };

    Json(json!({
        "status": status,
        "tasks": states,
    }))
}