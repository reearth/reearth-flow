import { Map as YMap } from "yjs";

import type { Edge, Node } from "@flow/types";

import { NonReactiveFields, YEdge, YNode } from "./workflowBuilder";

export const reassembleNode = (yNode: YNode): Node => {
  const id = yNode.get("id")?.toString() as string;
  const position = {
    x: (yNode.get("position") as YMap<any>).get("x"),
    y: (yNode.get("position") as YMap<any>).get("y"),
  };
  const measured = {
    width: (yNode.get("measured") as YMap<any>)?.get("width"),
    height: (yNode.get("measured") as YMap<any>)?.get("height"),
  };

  const style = {
    width: (yNode.get("style") as YMap<any>)?.get("width"),
    height: (yNode.get("style") as YMap<any>)?.get("height"),
  };

  // Get non-reactive fields
  const nonReactiveFields: NonReactiveFields = {
    // selected: yNode.get("selected") as boolean,
    dragging: yNode.get("dragging") as boolean,
    data: yNode.get("data") as any,
    type: yNode.get("type") as string,
  };

  const reassembledNode: Node = {
    id,
    position,
    type: nonReactiveFields["type"],
    dragging: nonReactiveFields["dragging"],
    measured,
    data: nonReactiveFields["data"],
  };

  if (nonReactiveFields["type"] === "batch" && style.width && style.height) {
    reassembledNode.style = style;
  }
  // if (nonReactiveFields["selected"]) {
  //   reassembledNode.selected = nonReactiveFields["selected"];
  // }

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
