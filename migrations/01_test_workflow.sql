INSERT INTO workflows.workflows (
    name
) 
VALUES 
    ('test-workflow-1'), 
    ('test-workflow-2')
ON CONFLICT (name) DO NOTHING;

INSERT INTO workflows.tasks (   
    workflow_id,
    task_idx,
    task_name, 
    command
)
VALUES
(
    1, 1, 'Task 1', 'echo Hello from Workflow 1 Task 1'
),
(
    1, 2, 'Task 2', 'echo Hello from Workflow 1 Task 2'
),
(
    1, 3, 'Task 3', 'echo Hello from Workflow 1 Task 3'
),
(
    1, 4, 'Task 4', 'echo Hello from Workflow 1 Task 4'
),
(
    2, 1, 'Task 1', 'echo Hello from Workflow 2 Task 1'
),
(
    2, 2, 'Task 2', 'echo Hello from Workflow 2 Task 2'
),
(
    2, 3, 'Task 3', 'echo Hello from Workflow 2 Task 3'
),
(
    2, 4, 'Task 4', 'echo Hello from Workflow 2 Task 4'
)
ON CONFLICT (workflow_id, task_idx) DO NOTHING;

INSERT INTO workflows.dependencies (
    workflow_id,
    from_task_idx,
    to_task_idx
)
VALUES
    (1,1,3),
    (1,2,3),
    (1,3,4)
ON CONFLICT (workflow_id, from_task_idx, to_task_idx) DO NOTHING;