use crate::workflow::Workflow;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json, Extension};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;


pub async fn list_tasks(
    Extension(workflows): Extension<Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>>>,
) -> impl IntoResponse {
    let workflows = workflows.lock().await;
    
    let mut all_tasks = Vec::new();

    for (i, wf) in workflows.iter().enumerate() {
        let wf = wf.lock().await;
        let tasks: Vec<_> = wf.graph.node_weights().map(|task| {
            json!({
                "workflow_id": i,
                "task_id": task.id,
                "name": task.name,
                "state": format!("{:?}", task.state),
            })
        }).collect();

        all_tasks.extend(tasks);
    }

    Json(all_tasks)
}


pub async fn get_task(
    Path((workflow_id, task_id)): Path<(usize, usize)>,
    Extension(workflows): Extension<Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>>>,
) -> impl IntoResponse {
    let workflows = workflows.lock().await;

    if let Some(workflow) = workflows.get(workflow_id) {
        let workflow = workflow.lock().await;
        if let Some(task) = workflow.graph.node_weights().find(|task| task.id == task_id) {
            return Json(json!({
                "workflow_id": workflow_id,
                "task_id": task.id,
                "name": task.name,
                "command": task.command,
                "state": format!("{:?}", task.state),
            }))
            .into_response();
        }
    }

    (StatusCode::NOT_FOUND, Json(json!({ "error": "Task not found" }))).into_response()
}


// Get the status of the entire workflow
pub async fn get_workflow_status(
    Path(workflow_id): Path<usize>,
    Extension(workflows): Extension<Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>>>,
) -> impl IntoResponse {
    let workflows = workflows.lock().await;

    if let Some(workflow) = workflows.get(workflow_id) {
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

        return Json(json!({
            "workflow_id": workflow_id,
            "status": status,
            "tasks": states,
        }))
        .into_response();
    }

    (StatusCode::NOT_FOUND, Json(json!({ "error": "Workflow not found" }))).into_response()
}