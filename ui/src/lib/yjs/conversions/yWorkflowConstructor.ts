import * as Y from "yjs";

import { toYjsArray, toYjsMap, toYjsText } from "@flow/lib/yjs/conversions";
import type {
  YEdge,
  YEdgeValue,
  YNode,
  YNodeValue,
  YWorkflow,
} from "@flow/lib/yjs/types";
import type { Edge, Node } from "@flow/types";

export const yNodeConstructor = (node: Node): YNode => {
  const yNode = toYjsMap<YNodeValue>({
    id: toYjsText(node.id),
    type: toYjsText(node.type),
    dragging: false,
    position: toYjsMap(node.position),
    measured: toYjsMap(node.measured),
    parentId: toYjsText(node.parentId),
    // Reference src/types/node.ts for the NodeData type
    data: toYjsMap({
      officialName: toYjsText(node.data.officialName),
      customName: toYjsText(node.data.customName),
      inputs: toYjsArray(node.data.inputs?.map((input) => toYjsText(input))),
      outputs: toYjsArray(
        node.data.outputs?.map((output) => toYjsText(output)),
      ),
      status: toYjsText(node.data.status),
      params: node.data.params,
      // Subworkflow specific
      pseudoInputs: toYjsArray(
        node.data.pseudoInputs?.map((pseudoInput) => {
          const yPseudoInput = new Y.Map();
          yPseudoInput.set("nodeId", toYjsText(pseudoInput.nodeId));
          yPseudoInput.set("portName", toYjsText(pseudoInput.portName));
          return yPseudoInput;
        }),
      ),
      pseudoOutputs: toYjsArray(
        node.data.pseudoOutputs?.map((pseudoOutput) => {
          const yPseudoOutput = new Y.Map();
          yPseudoOutput.set("nodeId", toYjsText(pseudoOutput.nodeId));
          yPseudoOutput.set("portName", toYjsText(pseudoOutput.portName));
          return yPseudoOutput;
        }),
      ),
      // Batch & Note specific
      content: toYjsText(node.data.content),
      backgroundColor: toYjsText(node.data.backgroundColor),
      textColor: toYjsText(node.data.textColor),
    }),
    style: toYjsMap({
      width: node.style?.width,
      height: node.style?.height,
    }),
  }) as YNode;

  // TODO: figure out how to handle locking

  return yNode;
};

export const yEdgeConstructor = (edge: Edge): YEdge => {
  const yEdge = toYjsMap<YEdgeValue>({
    id: toYjsText(edge.id),
    source: toYjsText(edge.source),
    target: toYjsText(edge.target),
    sourceHandle: toYjsText(edge.sourceHandle),
    targetHandle: toYjsText(edge.targetHandle),
  }) as YEdge;

  return yEdge;
};

export const yWorkflowConstructor = (
  id: string,
  name: string,
  nodes?: Node[],
  edges?: Edge[],
) => {
  const yWorkflow = new Y.Map() as YWorkflow;
  const yId = toYjsText(id) ?? new Y.Text();
  const yName = toYjsText(name) ?? new Y.Text();
  const yNodes =
    toYjsArray<YNode>(nodes?.map((n) => yNodeConstructor(n))) ??
    new Y.Array<YNode>();
  const yEdges =
    toYjsArray<YEdge>(edges?.map((e) => yEdgeConstructor(e))) ??
    new Y.Array<YEdge>();

  yWorkflow.set("id", yId);
  yWorkflow.set("name", yName);
  yWorkflow.set("nodes", yNodes);
  yWorkflow.set("edges", yEdges);
  return yWorkflow;
};
