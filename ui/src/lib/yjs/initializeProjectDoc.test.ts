import * as Y from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { yWorkflowConstructor } from "./conversions";
import { initializeProjectDoc } from "./initializeProjectDoc";

const watchUpdates = (yDoc: Y.Doc) => {
  const updates = vi.fn();
  yDoc.on("update", updates);
  return updates;
};

describe("initializeProjectDoc", () => {
  test("gives a never-opened project its Main Workflow", () => {
    const yDoc = new Y.Doc();

    initializeProjectDoc(yDoc);

    expect(yDoc.getMap("metadata").get("initialized")).toBe(true);
    expect(yDoc.getMap("workflows").has(DEFAULT_ENTRY_GRAPH_ID)).toBe(true);
  });

  test("writes nothing once the project is initialised", () => {
    const yDoc = new Y.Doc();
    initializeProjectDoc(yDoc);
    const updates = watchUpdates(yDoc);

    initializeProjectDoc(yDoc);

    expect(updates).not.toHaveBeenCalled();
  });

  test("writes nothing to an older project that has a Main Workflow but no flag", () => {
    const yDoc = new Y.Doc();
    yDoc
      .getMap("workflows")
      .set(
        DEFAULT_ENTRY_GRAPH_ID,
        yWorkflowConstructor(DEFAULT_ENTRY_GRAPH_ID, "Main Workflow"),
      );
    const updates = watchUpdates(yDoc);

    initializeProjectDoc(yDoc);

    expect(updates).not.toHaveBeenCalled();
  });
});
