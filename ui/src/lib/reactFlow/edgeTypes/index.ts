import { EdgeTypes } from "@xyflow/react";
import { createElement } from "react";

import DefaultEdge, { CustomEdgeProps } from "./DefaultEdge";
import SimpleEdge from "./SimpleEdge";

export const edgeTypes: EdgeTypes = {
  default: DefaultEdge,
  simpleEdge: SimpleEdge,
};

export const createFullEdgeTypes = (workflowChain?: string): EdgeTypes => ({
  default: (props: CustomEdgeProps) =>
    createElement(DefaultEdge, { ...props, workflowChain }),
});

export const simpleEdgeTypes: EdgeTypes = {
  default: SimpleEdge,
};
