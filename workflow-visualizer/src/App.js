import React, { useEffect, useState } from "react";
import * as d3 from "d3";
import dagre from "dagre";

const App = () => {
  const [graphData, setGraphData] = useState(null);

  fetch("http://localhost:3000/workflow/graph")
  .then((response) => response.json())
  .then((data) => {
    console.log(data); // Log the response to ensure it's correct
    setGraphData(data.workflows[0]);
  })
  .catch((error) => console.error("Fetch error:", error));

  useEffect(() => {
    if (graphData) {
      renderGraph(graphData);
    }
  }, [graphData]);

  const renderGraph = (data) => {
    const g = new dagre.graphlib.Graph();
    g.setGraph({});
    g.setDefaultEdgeLabel(() => ({}));

    // Add nodes
    data.nodes.forEach((node) => {
      g.setNode(node.id, { label: node.name, width: 100, height: 50 });
    });

    // Add edges
    data.edges.forEach((edge) => {
      g.setEdge(edge.from, edge.to);
    });

    dagre.layout(g);

    const svg = d3.select("svg");
    svg.selectAll("*").remove(); // Clear existing content

    const container = svg.append("g");

    // Render nodes
    g.nodes().forEach((v) => {
      const node = g.node(v);
      container
        .append("rect")
        .attr("x", node.x - 50)
        .attr("y", node.y - 25)
        .attr("width", node.width)
        .attr("height", node.height)
        .attr("fill", "#61dafb")
        .attr("stroke", "#333");

      container
        .append("text")
        .attr("x", node.x)
        .attr("y", node.y)
        .attr("dy", 5)
        .attr("text-anchor", "middle")
        .text(node.label);
    });

    // Render edges
    g.edges().forEach((e) => {
      const edge = g.edge(e);
      container
        .append("line")
        .attr("x1", edge.points[0].x)
        .attr("y1", edge.points[0].y)
        .attr("x2", edge.points[edge.points.length - 1].x)
        .attr("y2", edge.points[edge.points.length - 1].y)
        .attr("stroke", "#999")
        .attr("stroke-width", 2);
    });
  };

  return (
    <div>
      <h1>Workflow Graph Visualization</h1>
      <svg width="800" height="600"></svg>
    </div>
  );
};

export default App;
