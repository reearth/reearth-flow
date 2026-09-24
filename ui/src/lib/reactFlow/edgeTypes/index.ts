import type { EdgeTypes } from "@xyflow/react";
import { createElement } from "react";

import type { CustomEdgeProps } from "./DefaultEdge";
import DefaultEdge from "./DefaultEdge";
import SimpleEdge from "./SimpleEdge";

export const edgeTypes: EdgeTypes = {
  default: DefaultEdge,
  simpleEdge: SimpleEdge,
};

export const createFullEdgeTypes = (currentWorkflowId?: string): EdgeTypes => ({
  default: (props: CustomEdgeProps) =>
    createElement(DefaultEdge, { ...props, currentWorkflowId }),
});

export const simpleEdgeTypes: EdgeTypes = {
  default: SimpleEdge,
};
