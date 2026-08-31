import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import type { YEdge, YNode, YNodeValue } from "../types";

import { toYjsArray, toYjsMap, toYjsText } from "./sharedTypes";

/**
 * Updating an entry of a YNodesMap/YEdgesMap must never be done by `set`ting a
 * freshly built Y.Map over the existing key. In a Y.Map a `set` beats a
 * concurrent `delete`, so rebuilding a node/edge that another client is
 * deleting at the same time resurrects it — the node comes back while whatever
 * else that delete removed (its edges, its subworkflow graph) stays gone. That
 * is the "ghost edge" class of bug.
 *
 * Mutating the fields of the existing Y.Map instead lets the delete win: the
 * writes land inside a type that was removed and are discarded on merge.
 *
 * The rule these helpers exist to enforce: only ever `set` a key on a
 * nodes/edges map to CREATE it. To update, mutate in place.
 */

// Assigns onto an existing map, removing the key when the value is absent so a
// field that has been cleared (e.g. a node leaving a batch loses `parentId`)
// does not linger.
const setOrDelete = (yMap: Y.Map<any>, key: string, value: unknown) => {
  if (value === null || value === undefined) {
    if (yMap.has(key)) yMap.delete(key);
    return;
  }
  yMap.set(key, value);
};

// Mutates a nested Y.Map (position/measured/style) rather than replacing it,
// so concurrent edits to sibling fields survive.
const updateNestedMap = (
  parent: Y.Map<any>,
  key: string,
  values: Record<string, unknown>,
) => {
  const existing = parent.get(key) as Y.Map<any> | undefined;
  if (!(existing instanceof Y.Map)) {
    setOrDelete(parent, key, toYjsMap(values));
    return;
  }
  Object.entries(values).forEach(([k, v]) => setOrDelete(existing, k, v));
};

export const updateYEdge = (yEdge: YEdge, edge: Edge) => {
  // `id` is the identity of the entry and matches its key — never rewritten.
  setOrDelete(yEdge, "source", toYjsText(edge.source));
  setOrDelete(yEdge, "target", toYjsText(edge.target));
  setOrDelete(yEdge, "sourceHandle", toYjsText(edge.sourceHandle));
  setOrDelete(yEdge, "targetHandle", toYjsText(edge.targetHandle));
};

export const updateYNodePosition = (
  yNode: YNode,
  position: { x: number; y: number },
) => updateNestedMap(yNode, "position", { x: position.x, y: position.y });

const toYPseudoPorts = (ports?: { nodeId: string; portName: string }[]) =>
  toYjsArray(
    ports?.map((port) => {
      const yPort = new Y.Map();
      yPort.set("nodeId", toYjsText(port.nodeId));
      yPort.set("portName", toYjsText(port.portName));
      return yPort;
    }),
  );

// Mirrors the `data` shape built by yNodeConstructor.
const updateYNodeData = (yNode: YNode, node: Node) => {
  const existing = yNode.get("data") as Y.Map<YNodeValue> | undefined;
  if (!(existing instanceof Y.Map)) return;
  const { data } = node;

  setOrDelete(existing, "officialName", toYjsText(data.officialName));
  setOrDelete(
    existing,
    "inputs",
    toYjsArray(data.inputs?.map((input) => toYjsText(input))),
  );
  setOrDelete(
    existing,
    "outputs",
    toYjsArray(data.outputs?.map((output) => toYjsText(output))),
  );
  setOrDelete(existing, "params", data.params);
  setOrDelete(existing, "paramsSchema", data.paramsSchema);
  setOrDelete(existing, "customizations", data.customizations);
  setOrDelete(existing, "nodeMetadata", data.nodeMetadata);
  existing.set("isCollapsed", data.isCollapsed ?? false);
  existing.set("isDisabled", data.isDisabled ?? false);
  setOrDelete(existing, "workflowPath", toYjsText(data.workflowPath));
  setOrDelete(
    existing,
    "subworkflowId",
    node.type === "subworkflow"
      ? toYjsText(data.subworkflowId ?? node.id)
      : undefined,
  );
  setOrDelete(existing, "pseudoInputs", toYPseudoPorts(data.pseudoInputs));
  setOrDelete(existing, "pseudoOutputs", toYPseudoPorts(data.pseudoOutputs));
};

export const updateYNode = (yNode: YNode, node: Node) => {
  // `id` is the identity of the entry and matches its key — never rewritten.
  setOrDelete(yNode, "type", toYjsText(node.type));
  setOrDelete(yNode, "parentId", toYjsText(node.parentId));
  yNode.set("dragging", false);
  updateYNodePosition(yNode, {
    x: node.position?.x ?? 0,
    y: node.position?.y ?? 0,
  });
  updateNestedMap(yNode, "measured", {
    width: node.measured?.width ?? 0,
    height: node.measured?.height ?? 0,
  });
  updateNestedMap(yNode, "style", {
    width: node.style?.width,
    height: node.style?.height,
  });
  updateYNodeData(yNode, node);
};
