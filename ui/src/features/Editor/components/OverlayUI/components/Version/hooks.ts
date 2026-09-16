import { useCallback, useRef, useState } from "react";
import * as Y from "yjs";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useDocument } from "@flow/lib/gql/document/useApi";
import { useT } from "@flow/lib/i18n";
import type { YWorkflow } from "@flow/lib/yjs/types";

import { docFromUpdate, makeGetMetadata, revertUpdate } from "./yjsRevert";

// Preview and restore are keyed by snapshotNumber via projectNamedSnapshot; restore
// applies an inverse update rather than calling rollbackProject, which would prune.
export default ({
  projectId,
  yDoc,
  onDialogClose,
}: {
  projectId: string;
  yDoc: Y.Doc | null;
  onDialogClose?: () => void;
}) => {
  const {
    useGetProjectNamedSnapshots,
    useGetLatestProjectSnapshot,
    useGetProjectNamedSnapshot,
    useSaveNamedSnapshot,
  } = useDocument();

  const { snapshots, isFetching, isError } =
    useGetProjectNamedSnapshots(projectId);
  const { projectDocument } = useGetLatestProjectSnapshot(projectId);
  const latestProjectSnapshotVersion = projectDocument;

  const { toast } = useToast();
  const t = useT();

  const previewDocRef = useRef<Y.Doc | null>(null);
  const [previewDocYWorkflows, setPreviewDocYWorkflows] =
    useState<Y.Map<YWorkflow> | null>(null);
  const [selectedSnapshotNumber, setSelectedSnapshotNumber] = useState<
    number | null
  >(null);
  const [isLoadingPreview, setIsLoadingPreview] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [openVersionConfirmationDialog, setOpenVersionConfirmationDialog] =
    useState(false);

  const destroyPreview = useCallback(() => {
    previewDocRef.current?.destroy();
    previewDocRef.current = null;
    setPreviewDocYWorkflows(null);
  }, []);

  const onSnapshotSelect = useCallback(
    async (snapshotNumber: number) => {
      setSelectedSnapshotNumber(snapshotNumber);
      destroyPreview();
      setIsLoadingPreview(true);

      try {
        const updates = await useGetProjectNamedSnapshot(
          projectId,
          snapshotNumber,
        );
        if (!updates?.length) {
          // Retention evicts snapshots, so a listed row can be gone by the time
          // it is clicked. Say so rather than showing an unchanged canvas.
          setSelectedSnapshotNumber(null);
          return toast({
            title: t("Version unavailable"),
            description: t("This version is no longer available."),
            variant: "destructive",
          });
        }

        const previewDoc = docFromUpdate(updates, "snapshot-preview");
        previewDocRef.current = previewDoc;
        setPreviewDocYWorkflows(previewDoc.getMap<YWorkflow>("workflows"));
      } catch (error) {
        console.error("Snapshot preview failed:", error);
        setSelectedSnapshotNumber(null);
        toast({
          title: t("Could not load this version"),
          variant: "destructive",
        });
      } finally {
        setIsLoadingPreview(false);
      }
    },
    [projectId, useGetProjectNamedSnapshot, destroyPreview, toast, t],
  );

  const onSnapshotRestore = useCallback(async () => {
    if (selectedSnapshotNumber === null || !yDoc) return;
    setIsRestoring(true);

    try {
      const updates = await useGetProjectNamedSnapshot(
        projectId,
        selectedSnapshotNumber,
      );
      if (!updates?.length) {
        throw new Error(`snapshot ${selectedSnapshotNumber} has no state`);
      }

      // Snapshot the current state FIRST, so the state being replaced stays
      // reachable from this panel. Auto-versioning is only every 15 minutes, so
      // recent work may otherwise have no snapshot of its own.
      await useSaveNamedSnapshot(projectId, t("Before restore"));

      // Additive, not a prune: this leaves every existing update in place and
      // reaches other peers as an ordinary edit.
      yDoc.transact(() => {
        revertUpdate(yDoc, updates, makeGetMetadata(yDoc));
      });

      setOpenVersionConfirmationDialog(false);
      onDialogClose?.();
    } catch (error) {
      console.error("Snapshot restore failed:", error);
      setOpenVersionConfirmationDialog(false);
      toast({
        title: t("Restore failed"),
        description: t("This version could not be restored."),
        variant: "destructive",
      });
    } finally {
      setIsRestoring(false);
    }
  }, [
    projectId,
    selectedSnapshotNumber,
    yDoc,
    useGetProjectNamedSnapshot,
    useSaveNamedSnapshot,
    onDialogClose,
    toast,
    t,
  ]);

  return {
    snapshots,
    latestProjectSnapshotVersion,
    isFetching,
    isError,
    previewDocRef,
    previewDocYWorkflows,
    selectedSnapshotNumber,
    isLoadingPreview,
    isRestoring,
    openVersionConfirmationDialog,
    setOpenVersionConfirmationDialog,
    onSnapshotSelect,
    onSnapshotRestore,
    destroyPreview,
  };
};
