import * as Y from "yjs";

export type YNodeValue = Y.Text | Y.Map<unknown> | number | boolean; // add other possible types

export type YNode = Y.Map<YNodeValue>;

export type YEdgeValue = Y.Text;

export type YEdge = Y.Map<YEdgeValue>;

export type YNodesArray = Y.Array<YNode>;

export type YEdgesArray = Y.Array<YEdge>;

export type YWorkflow = Y.Map<Y.Text | YNodesArray | YEdgesArray>;
