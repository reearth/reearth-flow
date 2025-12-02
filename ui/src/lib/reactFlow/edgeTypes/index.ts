import { EdgeTypes } from "@xyflow/react";

import DefaultEdge from "./DefaultEdge";
import SimpleEdge from "./SimpleEdge";

export const edgeTypes: EdgeTypes = {
  default: DefaultEdge,
  simpleEdge: SimpleEdge,
};

export const fullEdgeTypes: EdgeTypes = {
  default: DefaultEdge,
};

export const simpleEdgeTypes: EdgeTypes = {
  default: SimpleEdge,
};
