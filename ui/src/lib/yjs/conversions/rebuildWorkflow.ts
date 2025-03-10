import * as Y from "yjs";

import { Workflow } from "@flow/types";
import type { Edge, Node, NodeData, NodeType } from "@flow/types";

import type { YWorkflow, YEdge, YNode } from "../types";

export const reassembleNode = (yNode: YNode): Node => {
  const id = yNode.get("id")?.toString() as string;
  const position = {
    x: (yNode.get("position") as Y.Map<any>).get("x"),
    y: (yNode.get("position") as Y.Map<any>).get("y"),
  };
  const type = yNode.get("type")?.toString() as NodeType;
  const dragging = yNode.get("dragging") as boolean;
  const measured = {
    width: (yNode.get("measured") as Y.Map<any>)?.get("width"),
    height: (yNode.get("measured") as Y.Map<any>)?.get("height"),
  };
  const parentId = yNode.get("parentId")?.toString();

  const data: NodeData = {
    officialName: (yNode.get("data") as Y.Map<any>)
      ?.get("officialName")
      .toString(),
  };
  if ((yNode.get("data") as Y.Map<any>)?.get("customName") !== undefined) {
    data.customName = (yNode.get("data") as Y.Map<any>)
      ?.get("customName")
      .toString();
  }
  if ((yNode.get("data") as Y.Map<any>)?.get("inputs") !== undefined) {
    data.inputs = (
      (yNode.get("data") as Y.Map<any>)?.get("inputs").toArray() as Y.Text[]
    ).map((input) => input.toString());
  }
  if ((yNode.get("data") as Y.Map<any>)?.get("outputs") !== undefined) {
    data.outputs = (
      (yNode.get("data") as Y.Map<any>)?.get("outputs").toArray() as Y.Text[]
    ).map((input) => input.toString());
  }
  if ((yNode.get("data") as Y.Map<any>)?.get("params") !== undefined) {
    data.params = (yNode.get("data") as Y.Map<any>)?.get("params");
  }
  if ((yNode.get("data") as Y.Map<any>)?.get("customizations") !== undefined) {
    data.customizations = (yNode.get("data") as Y.Map<any>)?.get(
      "customizations",
    );
  }
  // Subworkflow specific
  if ((yNode.get("data") as Y.Map<any>)?.get("subworkflowId") !== undefined) {
    data.subworkflowId = (yNode.get("data") as Y.Map<any>)
      ?.get("subworkflowId")
      .toString();
  }
  if ((yNode.get("data") as Y.Map<any>)?.get("pseudoInputs") !== undefined) {
    data.pseudoInputs = (yNode.get("data") as Y.Map<any>)
      ?.get("pseudoInputs")
      .toArray()
      .map((input: Y.Map<any>) => ({
        nodeId: input.get("nodeId").toString(),
        portName: input.get("portName").toString(),
      }));
  }
  if ((yNode.get("data") as Y.Map<any>)?.get("pseudoOutputs") !== undefined) {
    data.pseudoOutputs = (yNode.get("data") as Y.Map<any>)
      ?.get("pseudoOutputs")
      .toArray()
      .map((input: Y.Map<any>) => ({
        nodeId: input.get("nodeId").toString(),
        portName: input.get("portName").toString(),
      }));
  }

  const style = {
    width: (yNode.get("style") as Y.Map<any>)?.get("width").toString(),
    height: (yNode.get("style") as Y.Map<any>)?.get("height").toString(),
  };

  const reassembledNode: Node = {
    id,
    position,
    measured,
    parentId,
    type,
    dragging,
    data,
  };

  if (type === "batch" && style.width && style.height) {
    reassembledNode.style = style;
  }

  return reassembledNode;
};

export const reassembleEdge = (yEdge: YEdge): Edge => {
  const id = yEdge.get("id")?.toString() as string;
  const source = yEdge.get("source")?.toString() as string;
  const target = yEdge.get("target")?.toString() as string;
  const sourceHandle = yEdge.get("sourceHandle")?.toString();
  const targetHandle = yEdge.get("targetHandle")?.toString();

  return {
    id,
    source,
    target,
    sourceHandle,
    targetHandle,
  };
};

export const rebuildWorkflow = (yWorkflow: YWorkflow): Workflow => {
  const workflow: Workflow = {
    id: "", // Default value, update if `id` is found in `yWorkflow`
  };

  // Iterate over the YWorkflow entries
  yWorkflow.forEach((value, key) => {
    if (key === "id" && value instanceof Y.Text) {
      workflow.id = value.toString();
    } else if (key === "name" && value instanceof Y.Text) {
      workflow.name = value.toString();
    } else if (key === "nodes" && value instanceof Y.Array) {
      // Convert nodes to plain objects
      workflow.nodes = value
        .toArray()
        .map((yNode) => reassembleNode(yNode as YNode));
    } else if (key === "edges" && value instanceof Y.Array) {
      // Convert edges to plain objects
      workflow.edges = value
        .toArray()
        .map((yEdge) => reassembleEdge(yEdge as YEdge));
    } else if (key === "createdAt" && value instanceof Y.Text) {
      workflow.createdAt = value.toString();
    } else if (key === "updatedAt" && value instanceof Y.Text) {
      workflow.updatedAt = value.toString();
    }
  });

  return workflow;
};
