import { Map as YMap, Text as YText, Array as YArray } from "yjs";

type YNodeValue = YText | YMap<unknown> | number | boolean; // add other possible types

export type YNode = YMap<YNodeValue>;

type YEdgeValue = YText;

export type YEdge = YMap<YEdgeValue>;

export type YNodesArray = YArray<YNode>;

export type YEdgesArray = YArray<YEdge>;

export type YWorkflow = YMap<YText | YNodesArray | YEdgesArray>;
