import {
  EngineReadyWorkflow,
  EngineReadyGraph,
  EngineReadyNode,
  EngineReadyEdge,
} from "@flow/types";

export class ValidationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ValidationError";
  }
}

export const validateEngineReadyEdge = (edge: any): edge is EngineReadyEdge => {
  return (
    typeof edge === "object" &&
    typeof edge.id === "string" &&
    typeof edge.from === "string" &&
    typeof edge.to === "string" &&
    typeof edge.fromPort === "string" &&
    typeof edge.toPort === "string"
  );
};

export const validateEngineReadyNode = (node: any): node is EngineReadyNode => {
  const hasRequiredProps =
    typeof node === "object" &&
    typeof node.id === "string" &&
    typeof node.name === "string" &&
    typeof node.type === "string";

  if (!hasRequiredProps) return false;

  // Optional properties type checking
  if ("subGraphId" in node && typeof node.subGraphId !== "string") return false;
  if ("action" in node && typeof node.action !== "string") return false;
  // 'with' can be any type so no validation needed

  return true;
};

export const validateEngineReadyGraph = (
  graph: any,
): graph is EngineReadyGraph => {
  if (
    typeof graph !== "object" ||
    typeof graph.id !== "string" ||
    typeof graph.name !== "string" ||
    !Array.isArray(graph.nodes) ||
    !Array.isArray(graph.edges)
  ) {
    return false;
  }

  // Validate all nodes
  const validNodes = graph.nodes.every(validateEngineReadyNode);
  if (!validNodes) return false;

  // Validate all edges
  const validEdges = graph.edges.every(validateEngineReadyEdge);
  if (!validEdges) return false;

  return true;
};

export const validateEngineReadyWorkflow = (
  workflow: any,
): workflow is EngineReadyWorkflow => {
  if (
    typeof workflow !== "object" ||
    typeof workflow.id !== "string" ||
    typeof workflow.name !== "string" ||
    typeof workflow.entryGraphId !== "string" ||
    !Array.isArray(workflow.graphs)
  ) {
    return false;
  }

  // Validate all graphs
  const validGraphs = workflow.graphs.every(validateEngineReadyGraph);
  if (!validGraphs) return false;

  return true;
};

export const validateWorkflowJson = (
  jsonString: string,
): { isValid: boolean; error?: string } => {
  let parsedJson: any;

  try {
    parsedJson = JSON.parse(jsonString);
  } catch (_e) {
    return {
      isValid: false,
      error: "Invalid JSON format",
    };
  }

  if (!validateEngineReadyWorkflow(parsedJson)) {
    return {
      isValid: false,
      error: "JSON does not match EngineReadyWorkflow structure",
    };
  }

  return { isValid: true };
};
