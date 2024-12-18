# Thermidor: Data Engineering Orchestration Software
![Thermidor Logo](img/thermidor.jpg)

**Thermidor** is a data orchestration tool written in **Rust**. It allows you to define, schedule, and execute data workflows with dependencies between tasks, ensuring efficient and reliable task execution.

Thermidor supports:

- **Parallel Task Execution** with dependency management.
- **Retry Logic** for tasks with configurable attempts.
- **State Persistence** to save and load workflow states.
- **HTTP API** to interact with workflows and tasks.

---

## Features

- **Directed Acyclic Graph (DAG) Execution**: Define tasks and dependencies in a graph structure.
- **Asynchronous Execution**: Run tasks concurrently using `tokio`.
- **Retry Mechanism**: Tasks retry on failure with configurable retries and timeouts.
- **State Tracking**: Track task states (`Pending`, `Running`, `Success`, `Failure`, `Skipped`).
- **HTTP API**: Query workflows, tasks, and their statuses via an HTTP API.

---

## Requirements

To run Thermidor, you need the following installed:

- **Rust** (1.60 or later): [Install Rust](https://www.rust-lang.org/tools/install)
- **Node.js and npm**: For the front-end application.
---

## Installation

Clone the repository:

```bash
git clone https://github.com/yourusername/thermidor.git
cd thermidor
```

Install dependencies:

```bash
cargo build
```

---

## Running the Application

### Start the Server

To start the Thermidor application and HTTP server, run:

```bash
cargo run
```

By default, the server runs on **`http://0.0.0.0:3000`**.

### Define Workflows

Workflows are defined in the `src/workflow.rs` file and saved as JSON. For example:

```json
[
    {"id": 1, "name": "Task 1", "command": "echo Hello from Task 1"},
    {"id": 2, "name": "Task 2", "command": "echo Hello from Task 2"}
]
```

Workflows and their dependencies can be configured in the `initialize_workflows` function within `workflow.rs`.

### API Endpoints

Once the server is running, use the following HTTP endpoints to interact with Thermidor:

1. **List All Tasks**:

   ```bash
   curl http://localhost:3000/workflows
   ```

2. **Get Task Details**:

   ```bash
   curl http://localhost:3000/workflow/{workflow_id}/task/{task_id}
   ```

3. **Get Workflow Status**:

   ```bash
   curl http://localhost:3000/workflow/{workflow_id}/status
   ```

4. **Get Workflow Graph**:

   ```bash
   curl http://localhost:3000/workflow/graph
   ```

---

## Example Workflow Execution

When you run `cargo run`, Thermidor will:

1. **Load or Create Workflows** from predefined tasks and dependencies.
2. **Schedule Tasks** based on dependencies.
3. **Execute Tasks** concurrently where possible.
4. **Retry Failed Tasks** up to the configured maximum attempts.
5. **Serve API Endpoints** for monitoring and querying workflows.

Sample output in the logs:

```
INFO thermidor::workflow: Loaded workflow from 'workflow1.json'
INFO thermidor::scheduler: Scheduling task: Task 1
INFO thermidor::scheduler: Scheduling task: Task 2
INFO thermidor::task: Executing task 1: Task 1
INFO thermidor::task: Executing task 2: Task 2
INFO thermidor::scheduler: Task 'Task 1' completed successfully.
INFO thermidor::scheduler: Task 'Task 2' completed successfully.
```

---

## Project Structure

```
thermidor/
├── Cargo.toml            # Project dependencies
└── src/
    ├── main.rs           # Entry point
    ├── scheduler.rs      # Task scheduler
    ├── task.rs           # Task definition and execution
    ├── workflow.rs       # Workflow creation and management
    ├── state.rs          # Task states
    └── api.rs            # HTTP API endpoints
```
