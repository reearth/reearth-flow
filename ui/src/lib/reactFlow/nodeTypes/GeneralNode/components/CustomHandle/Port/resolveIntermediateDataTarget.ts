import type { NodeData } from "@flow/types";

export type IntermediateDataTarget = {
  nodeId: string;
  portName: string;
  workflowPath: string;
};

/**
 * Where an output port's intermediate data actually lives.
 *
 * For most nodes the canvas coordinates and the engine coordinates are the
 * same. A subworkflow node is the exception: it stands in for a whole subgraph,
 * and its output handles are pseudo ports (`pseudoOutputs`) naming that
 * subgraph's Output Router nodes. The engine never writes a feature-store file
 * for the subworkflow node itself — it writes one for the Output Router, inside
 * the child workflow. So the artifact is at
 * `<parentPath>.<subworkflowId>.<routerNodeId>.<port>`, never at
 * `<parentPath>.<subworkflowNodeId>.<port>`.
 */
export const resolveIntermediateDataTarget = ({
  nodeId,
  nodeData,
  portName,
}: {
  nodeId: string;
  nodeData: NodeData;
  portName: string;
}): IntermediateDataTarget => {
  const workflowPath = nodeData.workflowPath ?? "";
  const { subworkflowId } = nodeData;

  const pseudoOutput = subworkflowId
    ? nodeData.pseudoOutputs?.find((po) => po.portName === portName)
    : undefined;

  if (!subworkflowId || !pseudoOutput) {
    return { nodeId, portName, workflowPath };
  }

  return {
    nodeId: pseudoOutput.nodeId,
    portName: pseudoOutput.portName,
    workflowPath: workflowPath
      ? `${workflowPath}.${subworkflowId}`
      : subworkflowId,
  };
};

/** Feature-store artifact URL for an already-resolved target. */
export const intermediateDataArtifactUrl = (
  api: string,
  jobId: string,
  { nodeId, portName, workflowPath }: IntermediateDataTarget,
): string =>
  `${api}/artifacts/${jobId}/feature-store/${workflowPath ? `${workflowPath}.` : ""}${nodeId}.${portName}.jsonl.zst`;
