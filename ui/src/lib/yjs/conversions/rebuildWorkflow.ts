import * as Y from "yjs";

import { ProjectCorruptionError } from "@flow/errors";
import { Workflow } from "@flow/types";
import type { Edge, Node, NodeData, NodeType } from "@flow/types";

import type { YWorkflow, YEdge, YNode, YNodesMap, YEdgesMap } from "../types";

export const reassembleNode = (yNode: YNode): Node => {
  const id = yNode.get("id")?.toString() as string;

  const positionMap = yNode.get("position") as Y.Map<any>;
  const position = {
    x: positionMap?.get("x") ?? 0,
    y: positionMap?.get("y") ?? 0,
  };

  if (
    positionMap &&
    (positionMap.get("x") == null || positionMap.get("y") == null)
  ) {
    console.warn(
      `Node ${yNode.get("id")} had null position, using default (0,0)`,
    );
  }
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
  if ((yNode.get("data") as Y.Map<any>)?.get("isCollapsed") !== undefined) {
    data.isCollapsed = (yNode.get("data") as Y.Map<any>)?.get("isCollapsed");
  }
  if ((yNode.get("data") as Y.Map<any>)?.get("isDisabled") !== undefined) {
    data.isDisabled = (yNode.get("data") as Y.Map<any>)?.get("isDisabled");
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
    } else if (key === "nodes" && value instanceof Y.Map) {
      // Convert map of nodes to array of plain objects

      try {
        workflow.nodes = Array.from(value as YNodesMap).map(([, yNode]) =>
          reassembleNode(yNode as YNode),
        );
      } catch {
        throw new ProjectCorruptionError(
          "Could not reassemble node. This project may be corrupted.",
        );
      }
    } else if (key === "edges" && value instanceof Y.Map) {
      // Convert map of edges to array of plain objects
      workflow.edges = Array.from(value as YEdgesMap).map(([, yEdge]) =>
        reassembleEdge(yEdge as YEdge),
      );
    } else if (key === "createdAt" && value instanceof Y.Text) {
      workflow.createdAt = value.toString();
    } else if (key === "updatedAt" && value instanceof Y.Text) {
      workflow.updatedAt = value.toString();
    }
  });

  return workflow;
};
