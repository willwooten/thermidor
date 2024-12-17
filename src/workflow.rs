use petgraph::graph::{DiGraph, NodeIndex};
use crate::task::Task;

pub struct Workflow {
    pub graph: DiGraph<Task, ()>,
}


impl Workflow {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
        }
    }
    /// Adds a task to the workflow and returns its NodeIndex.
    pub fn add_task(&mut self, task: Task) -> NodeIndex {
        self.graph.add_node(task)
    }

    /// Adds a dependency between two tasks.
    pub fn add_dependency(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ());
    }
}
