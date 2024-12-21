use crate::workflow::Workflow;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json, Extension, Router, routing::get};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use futures::future::join_all;
use tower_http::cors::{CorsLayer, AllowOrigin, Any};


pub fn create_app(workflows: Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>>) -> Router {
    Router::new()
        .route("/workflows", get(list_tasks))
        .route("/workflow/:workflow_id/task/:id", get(get_task))
        .route("/workflow/:workflow_id/status", get(get_workflow_status))
        .route("/workflow/graph", get(get_workflow_graph))
        .route("/workflow/:workflow_id/timeline", get(get_execution_timeline)) // Add this line
        .layer(Extension(workflows))
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::exact(
                    "http://localhost:3001".parse().unwrap(), // Allow only this specific origin
                ))
                .allow_methods(Any) // Allow all HTTP methods
                .allow_headers(Any), // Allow all headers
        )
}

pub async fn run_server(workflows: Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>>) {
    let app = create_app(workflows);

    let addr = "0.0.0.0:3000".parse().unwrap();
    info!("Listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


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

/// Generates a graph representation of all workflows.
///
/// This asynchronous function creates a graph structure for each workflow, consisting of tasks (nodes)
/// and their dependencies (edges). The function returns a JSON response containing the graph data
/// for all workflows.
///
/// # Arguments
/// - `Extension(workflows)`: An Axum `Extension` that provides a thread-safe, shared reference
///   to the vector of workflows (`Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>>>`).
///
/// # Returns
/// - `Json<serde_json::Value>`: A JSON response containing the graph data of all workflows.
///
/// # Example Response
/// ```json
/// {
///     "workflows": [
///         {
///             "workflow_id": 0,
///             "nodes": [
///                 { "id": 1, "name": "Task 1" },
///                 { "id": 2, "name": "Task 2" }
///             ],
///             "edges": [
///                 { "from": 1, "to": 2 }
///             ]
///         }
///     ]
/// }
/// ```
pub async fn get_workflow_graph(
    Extension(workflows): Extension<Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>>>,
) -> Json<serde_json::Value> {
    let workflows = workflows.lock().await;

    // Create a vector of futures to be awaited
    let futures: Vec<_> = workflows.iter().enumerate().map(|(i, wf)| {
        let wf = Arc::clone(wf);
        async move {
            let wf_guard = wf.lock().await;
            let nodes: Vec<_> = wf_guard.graph.node_indices().map(|idx| {
                let task = &wf_guard.graph[idx];
                json!({
                    "id": task.id,
                    "name": task.name,
                })
            }).collect();

            let edges: Vec<_> = wf_guard.graph.edge_indices().map(|edge| {
                let (source, target) = wf_guard.graph.edge_endpoints(edge).unwrap();
                json!({
                    "from": wf_guard.graph[source].id,
                    "to": wf_guard.graph[target].id,
                })
            }).collect();

            json!({
                "workflow_id": i,
                "nodes": nodes,
                "edges": edges,
            })
        }
    }).collect();

    // Await all the futures
    let graph_data = join_all(futures).await;

    Json(json!({ "workflows": graph_data }))
}


pub async fn get_execution_timeline(
    Path(workflow_id): Path<usize>,
    Extension(workflows): Extension<Arc<Mutex<Vec<Arc<Mutex<Workflow>>>>>>,
) -> impl IntoResponse {
    let workflows = workflows.lock().await;

    if let Some(workflow) = workflows.get(workflow_id) {
        let workflow = workflow.lock().await;
        let timeline: Vec<_> = workflow.graph.node_weights().map(|task| {
            json!({
                "task_id": task.id,
                "name": task.name,
                "start_time": task.start_time,
                "end_time": task.end_time,
                "duration": task.end_time.map(|end| (end - task.start_time.unwrap_or(end)).num_seconds())
            })
        }).collect();

        return Json(json!({ "workflow_id": workflow_id, "timeline": timeline })).into_response();
    }

    (StatusCode::NOT_FOUND, Json(json!({ "error": "Workflow not found" }))).into_response()
}
