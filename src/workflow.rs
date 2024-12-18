use petgraph::dot::{Dot, Config};
use petgraph::graph::{DiGraph, NodeIndex};
use crate::task::Task;
use std::fs::File;
use std::io::{self, Write, Read};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
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

    /// Exports the workflow to a DOT file.
    pub fn export_to_dot(&self, filename: &str) -> io::Result<()> {
        let dot = format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]));
        let mut file = File::create(filename)?;
        file.write_all(dot.as_bytes())
    }

    /// Saves the workflow to a JSON file.
    pub fn save_to_json(&self, filename: &str) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(filename)?;
        file.write_all(json.as_bytes())
    }

    /// Loads the workflow from a JSON file.
    pub fn load_from_json(filename: &str) -> io::Result<Self> {
        let mut file = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let workflow = serde_json::from_str(&content)?;
        Ok(workflow)
    }
}
