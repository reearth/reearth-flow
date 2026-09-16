import * as Y from "yjs";

export type SharedTypeName = "Text" | "Map" | "Array";

// Both the corruption-recovery flow and snapshot restore need this, and it is
// subtle enough that two copies would drift. Shared deliberately.

// makeGetMetadata reports how each top-level key is typed in the live doc, which
// revertUpdate needs to rebuild the same shared types on the snapshot doc.
export const makeGetMetadata =
  (doc: Y.Doc) =>
  (key: string): SharedTypeName => {
    const sharedType = doc.share.get(key);
    if (sharedType instanceof Y.Text) return "Text";
    if (sharedType instanceof Y.Map) return "Map";
    if (sharedType instanceof Y.Array) return "Array";

    console.warn(`Could not determine type for ${key}, defaulting to Map`);
    return "Map";
  };

// docFromUpdate materializes a detached doc from a state update, for previewing
// without touching the live document.
export const docFromUpdate = (update: Uint8Array, origin: string): Y.Doc => {
  const doc = new Y.Doc();
  Y.applyUpdate(doc, update, origin);
  return doc;
};

// revertUpdate moves `doc` back to the state in `snapshotUpdate` by applying a
// forward update that undoes everything since, rather than deleting history.
//
// That distinction is the whole point: the result is an ordinary Yjs update on
// the live doc, so the websocket provider broadcasts it like any edit and peers
// converge on their own. No server-side prune, no forced resync, and the state
// being left behind stays in the update log.
//
// Adapted from https://discuss.yjs.dev/t/is-there-a-way-to-revert-to-a-specific-version/379/6
export const revertUpdate = (
  doc: Y.Doc,
  snapshotUpdate: Uint8Array,
  getMetadata: (key: string) => SharedTypeName,
  origin = "snapshot-rollback",
) => {
  const snapshotDoc = new Y.Doc();
  Y.applyUpdate(snapshotDoc, snapshotUpdate, origin);

  const currentStateVector = Y.encodeStateVector(doc);
  const snapshotStateVector = Y.encodeStateVector(snapshotDoc);
  const changesSinceSnapshotUpdate = Y.encodeStateAsUpdate(
    doc,
    snapshotStateVector,
  );

  const undoManager = new Y.UndoManager(
    [...snapshotDoc.share.keys()].map((key) => {
      const type = getMetadata(key);
      if (type === "Text") return snapshotDoc.getText(key);
      if (type === "Map") return snapshotDoc.getMap(key);
      if (type === "Array") return snapshotDoc.getArray(key);
      throw new Error("Unknown type");
    }),
    { trackedOrigins: new Set([origin]) },
  );

  Y.applyUpdate(snapshotDoc, changesSinceSnapshotUpdate, origin);
  undoManager.undo();

  const revertChangesSinceSnapshotUpdate = Y.encodeStateAsUpdate(
    snapshotDoc,
    currentStateVector,
  );
  Y.applyUpdate(doc, revertChangesSinceSnapshotUpdate, origin);
};
