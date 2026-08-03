import { useCallback, useRef, useState } from "react";
import { Doc } from "yjs";
import * as Y from "yjs";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useDocument } from "@flow/lib/gql/document/useApi";
import { useT } from "@flow/lib/i18n";
import type { YWorkflow } from "@flow/lib/yjs/types";

export default ({
  projectId,
  yDoc,
  onDialogClose,
}: {
  projectId: string;
  yDoc: Doc | null;
  onDialogClose: () => void;
}) => {
  const {
    useGetProjectSnapshots,
    useGetLatestProjectSnapshot,
    useRollbackProject,
    useGetPreviewProjectSnapshot,
  } = useDocument();
  // Version history is snapshot-backed. The raw CRDT update log is a
  // durability concern and is deliberately not shown: it has one entry per
  // flush.
  const { snapshots, isFetching } = useGetProjectSnapshots(projectId);
  const { projectDocument } = useGetLatestProjectSnapshot(projectId);
  const [selectedProjectSnapshotVersion, setSelectedProjectSnapshotVersion] =
    useState<number | null>(null);

  const previewDocRef = useRef<Y.Doc | null>(null);

  const [openVersionConfirmationDialog, setOpenVersionConfirmationDialog] =
    useState<boolean>(false);
  const [isReverting, setIsReverting] = useState<boolean>(false);
  const [isLoadingPreview, setIsLoadingPreview] = useState<boolean>(false);
  const [previewDocYWorkflows, setPreviewDocYWorkflows] =
    useState<Y.Map<YWorkflow> | null>(null);
  const lastErroredVersionRef = useRef<number | null>(null);
  const [isCorruptedVersion, setIsCorruptedVersion] = useState<boolean>(false);

  const { toast } = useToast();
  const t = useT();
  const latestProjectSnapshotVersion = projectDocument;
  // Note: This function comes from this forum: https://discuss.yjs.dev/t/is-there-a-way-to-revert-to-a-specific-version/379/6
  function revertUpdate(
    doc: Y.Doc,
    snapshotUpdate: Uint8Array,
    getMetadata: (key: string) => "Text" | "Map" | "Array",
  ) {
    const snapshotDoc = new Y.Doc();
    Y.applyUpdate(snapshotDoc, snapshotUpdate, "snapshot-rollback");

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
        trackedOrigins: new Set(["snapshot-rollback"]),
      },
    );
    Y.applyUpdate(snapshotDoc, changesSinceSnapshotUpdate, "snapshot-rollback");
    undoManager.undo();
    const revertChangesSinceSnapshotUpdate = Y.encodeStateAsUpdate(
      snapshotDoc,
      currentStateVector,
    );
    Y.applyUpdate(doc, revertChangesSinceSnapshotUpdate, "snapshot-rollback");
  }

  const handleProjectRollback = useCallback(async () => {
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
      setOpenVersionConfirmationDialog(false);
      onDialogClose();
    } catch (error) {
      console.error("Project Rollback Failed:", error);
      setOpenVersionConfirmationDialog(false);
      onDialogClose();
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
    projectId,
    yDoc,
    onDialogClose,
    t,
    toast,
    selectedProjectSnapshotVersion,
    useRollbackProject,
  ]);

  function createVersionPreview(snapshotUpdate: Uint8Array): Y.Doc {
    const snapshotDoc = new Y.Doc();
    Y.applyUpdate(snapshotDoc, snapshotUpdate, "snapshot-preview");
    return snapshotDoc;
  }

  // NOTE: `version` here is a NamedSnapshot.id from the snapshot list, not
  // the raw update-log version number that previewSnapshot/rollbackProject
  // were originally built for. The two are different, backend-assigned ID
  // spaces (see server/websocket-go/internal/gcs/snapshots.go, SnapNextIDName)
  // and there is currently no GraphQL field to preview or restore a
  // NamedSnapshot's own content by id. Until that's added, selecting a
  // snapshot row can preview/rollback to the wrong update-log entry if the
  // numbers happen to collide. Tracked as a follow-up alongside the
  // documented "restore semantics" gap; do not rely on Preview/Revert being
  // correct for snapshot-backed history until this is resolved.
  const handleVersionSelection = useCallback(
    async (version: number) => {
      if (isCorruptedVersion && version !== lastErroredVersionRef.current) {
        setIsCorruptedVersion(false);
      }

      setSelectedProjectSnapshotVersion(version);
      if (version === null) return;
      if (previewDocRef.current) {
        previewDocRef.current.destroy();
        previewDocRef.current = null;
      }
      setPreviewDocYWorkflows(null);
      setIsLoadingPreview(true);

      try {
        const previewData = await useGetPreviewProjectSnapshot(
          projectId,
          version,
        );
        if (!previewData) {
          console.error(
            "No project snapshot found for version: ",
            selectedProjectSnapshotVersion,
          );
          return;
        }
        const updates = previewData.previewSnapshot?.updates;
        if (!updates || !updates.length) {
          console.error("No updates found in snapshot");
          setIsLoadingPreview(false);
          return;
        }

        const convertedUpdates = new Uint8Array(updates);

        const versionPreviewYDoc = createVersionPreview(convertedUpdates);

        previewDocRef.current = versionPreviewYDoc;

        const versionpreviewPreviewYWorkflows =
          versionPreviewYDoc.getMap<YWorkflow>("workflows");

        if (!versionpreviewPreviewYWorkflows) {
          console.error("No workflows found in version preview");
          return;
        }

        setPreviewDocYWorkflows(versionpreviewPreviewYWorkflows);
      } catch (error) {
        console.error("Version Preview Failed:", error);
        return toast({
          title: t("Version Preview Failed"),
          description: t(
            "Project Version Preview cannot be viewed. An error has occurred.",
          ),
          variant: "destructive",
        });
      }
      setIsLoadingPreview(false);
    },
    [
      useGetPreviewProjectSnapshot,
      projectId,
      t,
      toast,
      isCorruptedVersion,
      selectedProjectSnapshotVersion,
    ],
  );

  const handleWorkflowCorruption = useCallback(() => {
    lastErroredVersionRef.current = selectedProjectSnapshotVersion;
    setIsCorruptedVersion(true);
  }, [
    selectedProjectSnapshotVersion,
    lastErroredVersionRef,
    setIsCorruptedVersion,
  ]);

  return {
    snapshots,
    latestProjectSnapshotVersion,
    previewDocRef,
    previewDocYWorkflows,
    selectedProjectSnapshotVersion,
    lastErroredVersionRef,
    isFetching,
    isLoadingPreview,
    isReverting,
    isCorruptedVersion,
    openVersionConfirmationDialog,
    setOpenVersionConfirmationDialog,
    setIsCorruptedVersion,
    onProjectRollback: handleProjectRollback,
    onVersionSelection: handleVersionSelection,
    onWorkflowCorruption: handleWorkflowCorruption,
  };
};
