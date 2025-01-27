import * as Y from "yjs";

import type {
  YEdge,
  YEdgesArray,
  YNode,
  YNodesArray,
  YWorkflow,
} from "@flow/lib/yjs/types";
import type { Edge, Node } from "@flow/types";

export const createYNode = (node: Node) => {
  const yNode = new Y.Map() as YNode;

  yNode.set("id", new Y.Text(node.id));
  yNode.set("type", new Y.Text(node.type));
  yNode.set("dragging", false);

  const yPosition = new Y.Map();
  yPosition.set("x", node.position.x);
  yPosition.set("y", node.position.y);
  yNode.set("position", yPosition);

  const yMeasured = new Y.Map();
  yMeasured.set("width", node.measured?.width);
  yMeasured.set("height", node.measured?.height);
  yNode.set("measured", yMeasured);

  // Reference src/types/node.ts for the NodeData type
  const yData = new Y.Map();
  yData.set("officialName", new Y.Text(node.data.officialName));
  if (node.data.customName) {
    yData.set("customName", new Y.Text(node.data.customName));
  }
  if (node.data.inputs) {
    const yInputs = new Y.Array();
    yInputs.insert(
      0,
      node.data.inputs.map((input) => new Y.Text(input)),
    );
    yData.set("inputs", yInputs);
  }
  if (node.data.outputs) {
    const yOutputs = new Y.Array();
    yOutputs.insert(
      0,
      node.data.outputs.map((output) => new Y.Text(output)),
    );
    yData.set("outputs", yOutputs);
  }
  if (node.data.status) {
    yData.set("status", new Y.Text(node.data.status));
  }
  if (node.data.params) {
    yData.set("params", node.data.params);
  }
  // Subworkflow specific
  if (node.data.pseudoInputs) {
    const yPseudoInputs = new Y.Array();
    yPseudoInputs.insert(
      0,
      node.data.pseudoInputs.map((pseudoInput) => {
        const yPseudoInput = new Y.Map();
        yPseudoInput.set("nodeId", new Y.Text(pseudoInput.nodeId));
        yPseudoInput.set("portName", new Y.Text(pseudoInput.portName));
        return yPseudoInput;
      }),
    );
    yData.set("pseudoInputs", yPseudoInputs);
  }
  if (node.data.pseudoOutputs) {
    const yPseudoOutputs = new Y.Array();
    yPseudoOutputs.insert(
      0,
      node.data.pseudoOutputs.map((pseudoOutput) => {
        const yPseudoOutput = new Y.Map();
        yPseudoOutput.set("nodeId", new Y.Text(pseudoOutput.nodeId));
        yPseudoOutput.set("portName", new Y.Text(pseudoOutput.portName));
        return yPseudoOutput;
      }),
    );
    yData.set("pseudoOutputs", yPseudoOutputs);
  }
  // Batch & Note specific
  if (node.data.content) {
    yData.set("content", new Y.Text(node.data.content));
  }
  if (node.data.backgroundColor) {
    yData.set("backgroundColor", new Y.Text(node.data.backgroundColor));
  }
  if (node.data.textColor) {
    yData.set("textColor", new Y.Text(node.data.textColor));
  }
  yNode.set("data", yData);

  if (node.type === "batch") {
    const yStyle = new Y.Map();
    yStyle.set("width", node.style?.width || 0);
    yStyle.set("height", node.style?.height || 0);
    yNode.set("style", yStyle);
  }

  // TODO: figure out how to handle locking

  return yNode;
};

export const createYEdge = (edge: Edge) => {
  const yEdge = new Y.Map() as YEdge;

  yEdge.set("id", new Y.Text(edge.id));
  yEdge.set("source", new Y.Text(edge.source));
  yEdge.set("target", new Y.Text(edge.target));
  if (edge.sourceHandle) {
    yEdge.set("sourceHandle", new Y.Text(edge.sourceHandle));
  }
  if (edge.targetHandle) {
    yEdge.set("targetHandle", new Y.Text(edge.targetHandle));
  }

  return yEdge;
};

export const yWorkflowBuilder = (
  id: string,
  name: string,
  nodes?: Node[],
  edges?: Edge[],
) => {
  const yWorkflow = new Y.Map() as YWorkflow;
  const yId = new Y.Text(id);
  const yName = new Y.Text(name);
  const yNodes = new Y.Array() as YNodesArray;
  const yEdges = new Y.Array() as YEdgesArray;

  if (nodes) {
    const yNodeMaps = nodes.map((node) => createYNode(node));
    yNodes.insert(0, yNodeMaps);
  }

  if (edges) {
    const yEdgeMaps = edges?.map((edge) => createYEdge(edge));
    yEdges.insert(0, yEdgeMaps);
  }

  yWorkflow.set("id", yId);
  yWorkflow.set("name", yName);
  yWorkflow.set("nodes", yNodes);
  yWorkflow.set("edges", yEdges);
  return yWorkflow;
};
