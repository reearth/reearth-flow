import * as Y from "yjs";

import type {
  YWorkflow as YWorkflowType,
  YEdgesMap,
  YNodesMap,
  YNode,
  YEdge,
} from "./types";

// TODO: integrate this into the system to remove the need for type assertions everywhere
export class YWorkflow {
  private map: YWorkflowType;

  constructor(map?: YWorkflowType) {
    // If no map is provided, create a new one.
    this.map = map ?? new Y.Map();

    // Ensure that all required keys are set.
    if (!this.map.get("id")) {
      this.map.set("id", new Y.Text());
    }
    if (!this.map.get("nodes")) {
      this.map.set("nodes", new Y.Map<YNode>());
    }
    if (!this.map.get("edges")) {
      this.map.set("edges", new Y.Map<YEdge>());
    }
  }

  getWorkflow(): YWorkflowType {
    return this.map;
  }

  get id(): Y.Text {
    // Since we've ensured a default above, this cast is safe.
    return this.map.get("id") as Y.Text;
  }

  get nodes(): YNodesMap {
    return this.map.get("nodes") as YNodesMap;
  }

  set nodes(nodes: YNodesMap) {
    this.map.set("nodes", nodes);
  }

  get edges(): YEdgesMap {
    return this.map.get("edges") as YEdgesMap;
  }

  set edges(edges: YEdgesMap) {
    this.map.set("edges", edges);
  }
}

export class YWorkflows {
  private map: Y.Map<YWorkflowType>;

  constructor(map?: Y.Map<YWorkflowType>) {
    // If no map is provided, create a new Y.Map for workflows.
    this.map = map ?? new Y.Map();
  }

  // Retrieve a Workflow by key. Wrap the underlying YWorkflow map in a Workflow instance.
  getWorkflow(key: string): YWorkflow | undefined {
    const wfMap = this.map.get(key);
    if (!wfMap) return undefined;
    return new YWorkflow(wfMap);
  }

  // Create (or update) a Workflow at the given key.
  setWorkflow(key: string, workflow?: YWorkflow): YWorkflow {
    // If no workflow is provided, create a new one.
    const wf = workflow ?? new YWorkflow();
    this.map.set(key, wf.getWorkflow());
    return wf;
  }

  // Iterate over all workflows.
  *values(): IterableIterator<YWorkflow> {
    for (const wfMap of this.map.values()) {
      yield new YWorkflow(wfMap);
    }
  }
}
