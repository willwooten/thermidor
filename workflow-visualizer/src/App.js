import React, { useEffect, useState, useCallback } from "react";
import * as d3 from "d3";
import dagre from "dagre";
import "./App.css";

const App = () => {
  const [graphData, setGraphData] = useState(null);

  useEffect(() => {
    fetch("http://localhost:3000/workflow/graph")
      .then((response) => response.json())
      .then((data) => {
        console.log(data); // Log the response to ensure it's correct
        setGraphData(data.workflows[0]);
      })
      .catch((error) => console.error("Fetch error:", error));
  }, []);

  // Create the tooltip once outside the renderGraph function
  const tooltip = d3.select("body")
    .append("div")
    .attr("class", "tooltip")
    .style("position", "absolute")
    .style("background", "#f9f9f9")
    .style("border", "1px solid #ccc")
    .style("padding", "8px")
    .style("border-radius", "5px")
    .style("pointer-events", "none")
    .style("opacity", 0);

  const renderGraph = useCallback((data) => {
    const g = new dagre.graphlib.Graph();
    g.setGraph({});
    g.setDefaultEdgeLabel(() => ({}));

    // Add nodes
    data.nodes.forEach((node) => {
      g.setNode(node.id, { label: node.name, width: 120, height: 60 });
    });

    // Add edges
    data.edges.forEach((edge) => {
      g.setEdge(edge.from, edge.to);
    });

    dagre.layout(g);

    const svg = d3.select("svg");
    svg.selectAll("*").remove(); // Clear existing content

    const container = svg.append("g");

    // Render edges
    g.edges().forEach((e) => {
      const edge = g.edge(e);
      container.append("path")
        .attr("class", "edge")
        .attr(
          "d",
          d3.line().curve(d3.curveBasis)(
            edge.points.map((p) => [p.x, p.y])
          )
        )
        .attr("stroke", "#666")
        .attr("stroke-width", 2)
        .attr("fill", "none");
    });

    // Render nodes
    const nodes = container.selectAll(".node")
      .data(g.nodes())
      .enter()
      .append("g")
      .attr("class", "node")
      .attr("transform", (v) => `translate(${g.node(v).x - 60}, ${g.node(v).y - 30})`);

    nodes.append("rect")
      .attr("width", 120)
      .attr("height", 60)
      .attr("rx", 8)
      .attr("ry", 8)
      .style("fill", "#61dafb")
      .style("stroke", "#2b6cb0")
      .on("mouseover", (event, v) => {
        const node = g.node(v);
        tooltip.transition().duration(200).style("opacity", 0.9);
        tooltip
          .html(`<strong>Task:</strong> ${node.label}<br><strong>ID:</strong> ${v}`)
          .style("left", `${event.pageX + 10}px`)
          .style("top", `${event.pageY - 20}px`);
      })
      .on("mouseout", () => {
        tooltip.transition().duration(200).style("opacity", 0);
      });

    nodes.append("text")
      .attr("x", 60)
      .attr("y", 30)
      .attr("dy", ".35em")
      .attr("text-anchor", "middle")
      .text((v) => g.node(v).label);
  }, [tooltip]);

  useEffect(() => {
    if (graphData) {
      renderGraph(graphData);
    }
  }, [graphData, renderGraph]);

  return (
    <div>
      <h1>Workflow Graph Visualization</h1>
      <svg width="100%" height="600" viewBox="0 0 800 600" preserveAspectRatio="xMidYMid meet"></svg>
    </div>
  );
};

export default App;
