import { useCallback, useState } from "react";
import { Doc } from "yjs";
import * as Y from "yjs";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useDocument } from "@flow/lib/gql/document/useApi";
import { useT } from "@flow/lib/i18n";

export default ({
  projectId,
  yDoc,
}: {
  projectId: string;
  yDoc: Doc | null;
}) => {
  const {
    useGetProjectHistory,
    useGetLatestProjectSnapshot,
    useRollbackProject,
  } = useDocument();

  const { history, isFetching } = useGetProjectHistory(projectId);

  const { projectDocument } = useGetLatestProjectSnapshot(projectId);

  const [selectedProjectSnapshotVersion, setSelectedProjectSnapshotVersion] =
    useState<number | null>(null);
  const [openVersionChangeDialog, setOpenVersionChangeDialog] =
    useState<boolean>(false);
  const [isReverting, setIsReverting] = useState<boolean>(false);
  const snapshotOrigin = "snapshot-rollback";
  const { toast } = useToast();
  const t = useT();
  // Note: This function comes from this forum: https://discuss.yjs.dev/t/is-there-a-way-to-revert-to-a-specific-version/379/6
  function revertUpdate(
    doc: Y.Doc,
    snapshotUpdate: Uint8Array,
    getMetadata: (key: string) => "Text" | "Map" | "Array",
  ) {
    const snapshotDoc = new Y.Doc();
    Y.applyUpdate(snapshotDoc, snapshotUpdate, snapshotOrigin);
    const currentStateVector = Y.encodeStateVector(doc);
    const snapshotStateVector = Y.encodeStateVector(snapshotDoc);
    const changesSinceSnapshotUpdate = Y.encodeStateAsUpdate(
      doc,
      snapshotStateVector,
    );
    const undoManager = new Y.UndoManager(
      [...snapshotDoc.share.keys()].map((key) => {
        const type = getMetadata(key);
        if (type === "Text") {
          return snapshotDoc.getText(key);
        } else if (type === "Map") {
          return snapshotDoc.getMap(key);
        } else if (type === "Array") {
          return snapshotDoc.getArray(key);
        }
        throw new Error("Unknown type");
      }),
      {
        trackedOrigins: new Set([snapshotOrigin]),
      },
    );
    Y.applyUpdate(snapshotDoc, changesSinceSnapshotUpdate, snapshotOrigin);
    undoManager.undo();
    const revertChangesSinceSnapshotUpdate = Y.encodeStateAsUpdate(
      snapshotDoc,
      currentStateVector,
    );
    Y.applyUpdate(doc, revertChangesSinceSnapshotUpdate, snapshotOrigin);
  }
  const handleRollbackProject = useCallback(async () => {
    if (selectedProjectSnapshotVersion === null) return;
    setIsReverting(true);
    try {
      const rollbackData = await useRollbackProject(
        projectId,
        selectedProjectSnapshotVersion,
      );

      const updates = rollbackData.projectDocument?.updates;

      if (!updates || !updates.length || !yDoc) {
        console.error("No updates found or yDoc not available");
        setIsReverting(false);
        return;
      }

      const convertedUpdates = new Uint8Array(updates);

      const getMetadata = (key: string): "Text" | "Map" | "Array" => {
        const sharedType = yDoc.share.get(key);
        if (sharedType instanceof Y.Text) return "Text";
        if (sharedType instanceof Y.Map) return "Map";
        if (sharedType instanceof Y.Array) return "Array";

        console.warn(`Could not determine type for ${key}, defaulting to Map`);
        return "Map";
      };

      yDoc.transact(() => {
        revertUpdate(yDoc, convertedUpdates, getMetadata);
      });
      setOpenVersionChangeDialog(false);
    } catch (error) {
      console.error("Project Rollback Failed:", error);
      setOpenVersionChangeDialog(false);
      return toast({
        title: t("Project Rollback Failed"),
        description: t(
          "Project cannot be rollbacked to this version. An error has occured.",
        ),
        variant: "destructive",
      });
    }
    setIsReverting(false);
  }, [
    selectedProjectSnapshotVersion,
    useRollbackProject,
    setIsReverting,
    projectId,
    yDoc,
    t,
    toast,
  ]);
  const latestProjectSnapshotVersion = projectDocument;
  return {
    history,
    isFetching,
    isReverting,
    latestProjectSnapshotVersion,
    selectedProjectSnapshotVersion,
    setSelectedProjectSnapshotVersion,
    openVersionChangeDialog,
    setOpenVersionChangeDialog,
    onRollbackProject: handleRollbackProject,
  };
};
