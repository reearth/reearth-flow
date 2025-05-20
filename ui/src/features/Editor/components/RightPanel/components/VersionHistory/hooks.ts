import { useCallback, useRef, useState } from "react";
import { Doc } from "yjs";
import * as Y from "yjs";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useDocument } from "@flow/lib/gql/document/useApi";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";

export default ({
  projectId,
  yDoc,
}: {
  projectId: string;
  yDoc: Doc | null;
}) => {
  const {
    useGetProjectHistory,
    useGetProjectSnapshot,
    useGetLatestProjectSnapshot,
    useRollbackProject,
  } = useDocument();
  const { history, isFetching } = useGetProjectHistory(projectId);
  const { projectDocument } = useGetLatestProjectSnapshot(projectId);
  const [selectedProjectSnapshotVersion, setSelectedProjectSnapshotVersion] =
    useState<number | null>(null);
  const { projectSnapshot } = useGetProjectSnapshot(
    projectId,
    selectedProjectSnapshotVersion,
  );
  const previewDocRef = useRef<Y.Doc | null>(null);

  const [openVersionChangeDialog, setOpenVersionChangeDialog] =
    useState<boolean>(false);
  const [openVersionPreviewDialog, setOpenVersionPreviewDialog] =
    useState<boolean>(false);
  const [isReverting, setIsReverting] = useState<boolean>(false);
  const [versionPreviewYWorkflows, setVersionPreviewYWorkflows] =
    useState<Y.Map<YWorkflow> | null>(null);
  const snapshotOriginRollback = "snapshot-rollback";
  const snapshotOriginPreview = "snapshot-preview";

  const { toast } = useToast();
  const t = useT();
  // Note: This function comes from this forum: https://discuss.yjs.dev/t/is-there-a-way-to-revert-to-a-specific-version/379/6
  function revertUpdate(
    doc: Y.Doc,
    snapshotUpdate: Uint8Array,
    getMetadata: (key: string) => "Text" | "Map" | "Array",
  ) {
    const snapshotDoc = new Y.Doc();
    Y.applyUpdate(snapshotDoc, snapshotUpdate, snapshotOriginRollback);

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
        trackedOrigins: new Set([snapshotOriginRollback]),
      },
    );
    Y.applyUpdate(
      snapshotDoc,
      changesSinceSnapshotUpdate,
      snapshotOriginRollback,
    );
    undoManager.undo();
    const revertChangesSinceSnapshotUpdate = Y.encodeStateAsUpdate(
      snapshotDoc,
      currentStateVector,
    );
    Y.applyUpdate(
      doc,
      revertChangesSinceSnapshotUpdate,
      snapshotOriginRollback,
    );
  }

  const handleRollbackProject = useCallback(async () => {
    if (selectedProjectSnapshotVersion === null) return;
    setIsReverting(true);
    try {
      const rollbackData = await useRollbackProject(
        projectId,
        selectedProjectSnapshotVersion,
      );

      console.log("TESTING ROLLBACK DATA", rollbackData);

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
      setOpenVersionPreviewDialog(false);
    } catch (error) {
      console.error("Project Rollback Failed:", error);
      setOpenVersionChangeDialog(false);
      setOpenVersionPreviewDialog(false);
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

  function createVersionPreview(
    snapshotUpdate: Uint8Array,
    getMetadata: (key: string) => "Text" | "Map" | "Array",
  ): Y.Doc {
    const snapshotDoc = new Y.Doc();
    Y.applyUpdate(snapshotDoc, snapshotUpdate, snapshotOriginPreview);

    for (const key of snapshotDoc.share.keys()) {
      const type = getMetadata(key);
      if (type === "Text") snapshotDoc.getText(key);
      else if (type === "Map") snapshotDoc.getMap(key);
      else if (type === "Array") snapshotDoc.getArray(key);
    }

    return snapshotDoc;
  }

  const handlePreviewVersion = useCallback(async () => {
    if (selectedProjectSnapshotVersion === null) return;

    try {
      if (!projectSnapshot) {
        console.error("No project snapshot found");
        return;
      }

      const updates = projectSnapshot.updates;

      if (!updates || !updates.length) {
        console.error("No updates found in snapshot");
        return;
      }

      const convertedUpdates = new Uint8Array(updates);

      const getMetadata = (key: string): "Text" | "Map" | "Array" => {
        const sharedType = yDoc?.share.get(key);
        if (sharedType instanceof Y.Text) return "Text";
        if (sharedType instanceof Y.Map) return "Map";
        if (sharedType instanceof Y.Array) return "Array";
        console.warn(`Unknown type for ${key}, defaulting to Map`);
        return "Map";
      };
      if (previewDocRef.current) {
        previewDocRef.current.destroy();
        previewDocRef.current = null;
      }

      const versionPreviewYDoc = createVersionPreview(
        convertedUpdates,
        getMetadata,
      );

      previewDocRef.current = versionPreviewYDoc;

      const maybeWorkflows = versionPreviewYDoc.share.get("workflows");

      if (!(maybeWorkflows instanceof Y.Map)) {
        console.error("No workflows found in version preview");
        return;
      }

      const versionPreviewYWorkflows = maybeWorkflows as Y.Map<YWorkflow>;

      console.log("Preview workflows map:", versionPreviewYWorkflows);
      setVersionPreviewYWorkflows(versionPreviewYWorkflows);
    } catch (error) {
      console.error("Project Version Preview Creation Failed:", error);
      return toast({
        title: t("Project Version Preview Creation"),
        description: t(
          "Project cannot be rolled back to this version. An error has occurred.",
        ),
        variant: "destructive",
      });
    }
  }, [selectedProjectSnapshotVersion, projectSnapshot, yDoc, t, toast]);

  return {
    history,
    isFetching,
    isReverting,
    latestProjectSnapshotVersion,
    selectedProjectSnapshotVersion,
    setSelectedProjectSnapshotVersion,
    versionPreviewYWorkflows,
    openVersionChangeDialog,
    openVersionPreviewDialog,
    setOpenVersionChangeDialog,
    setOpenVersionPreviewDialog,
    onRollbackProject: handleRollbackProject,
    onPreviewVersion: handlePreviewVersion,
  };
};
