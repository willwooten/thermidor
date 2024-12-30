DROP TABLE IF EXISTS workflows.task_status;
DROP TABLE IF EXISTS workflows.dependencies;
DROP TABLE IF EXISTS workflows.tasks;
DROP TABLE IF EXISTS workflows.workflows;
DROP SCHEMA IF EXISTS workflows;

CREATE SCHEMA IF NOT EXISTS workflows;

CREATE TABLE IF NOT EXISTS workflows.workflows (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    last_updated TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_workflows_workflows ON workflows.workflows(name);

CREATE TABLE IF NOT EXISTS workflows.tasks (
    workflow_id INTEGER REFERENCES workflows.workflows(id) ON DELETE CASCADE,
    task_idx INTEGER NOT NULL,
    task_name TEXT NOT NULL,
    command TEXT NOT NULL,
    retry_count INTEGER DEFAULT 0,
    timeout_duration INTERVAL DEFAULT INTERVAL '24 hours',
    start_time TIMESTAMP,
    end_time TIMESTAMP,
    status TEXT DEFAULT 'Pending',
    PRIMARY KEY (workflow_id, task_idx),
    UNIQUE (workflow_id, task_idx, task_name)
);

CREATE INDEX idx_workflows_tasks ON workflows.tasks(workflow_id, task_idx);

CREATE TABLE IF NOT EXISTS workflows.dependencies (
    workflow_id INTEGER REFERENCES workflows.workflows(id) ON DELETE CASCADE,
    from_task_idx INTEGER NOT NULL,
    to_task_idx INTEGER NOT NULL, 
    PRIMARY KEY (workflow_id, from_task_idx, to_task_idx),
    FOREIGN KEY (workflow_id, from_task_idx)
        REFERENCES workflows.tasks (workflow_id, task_idx) ON DELETE CASCADE,
    FOREIGN KEY (workflow_id, to_task_idx)
        REFERENCES workflows.tasks (workflow_id, task_idx) ON DELETE CASCADE
);

CREATE INDEX idx_workflows_dependencies ON workflows.dependencies(workflow_id);
