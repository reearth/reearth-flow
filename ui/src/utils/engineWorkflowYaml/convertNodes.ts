import {
  deployableNodeTypes,
  type EngineReadyNode,
  type Node,
} from "@flow/types";

import { isDefined } from "../isDefined";

export const convertNodes = (nodes?: Node[]) => {
  if (!nodes) return [];
  const convertedNodes: EngineReadyNode[] = nodes
    ?.map(({ id, type, data }) => {
      if (!id || !type || !data.name) return undefined;

      const n: EngineReadyNode = {
        id,
        name: data.name,
        type,
        // data, // TODO: conversion of data.params to with
      };
      if (type === "subworkflow") {
        n.subGraphId = id;
      }

      return n;
    })
    .filter(isDefined)
    .filter(isDeployable);
  return convertedNodes;
};

const isDeployable = (node: EngineReadyNode) =>
  node && deployableNodeTypes.includes(node.type);
