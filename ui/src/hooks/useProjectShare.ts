import { useCallback, useEffect, useMemo } from "react";
import { useY } from "react-yjs";
import { Doc, Map as YMap } from "yjs";

import { useSharedProject } from "@flow/lib/gql";
import { Project } from "@flow/types";

const emptyMetadata = new YMap();

export default ({
  currentProject,
  yDoc,
}: {
  currentProject?: Project | undefined;
  yDoc: Doc | null;
}) => {
  const { shareProject, unshareProject } = useSharedProject();

  const yMetadata = useMemo(() => yDoc?.getMap<any>("metadata"), [yDoc]);
  const metadata = useY(yMetadata ?? emptyMetadata);

  // Keep ydoc in sync with server
  useEffect(() => {
    if (!yMetadata) return;
    yMetadata.set("sharingToken", currentProject?.sharedToken ?? null);
  }, [yMetadata, currentProject?.sharedToken]);

  const sharingToken: string | undefined =
    "sharingToken" in (metadata ?? {})
      ? ((metadata.sharingToken as string | null) ?? undefined)
      : currentProject?.sharedToken;

  const sharingUrl = sharingToken
    ? `${window.location.origin}/shared/${sharingToken}`
    : undefined;

  const handleProjectShare = useCallback(
    async (share: boolean) => {
      if (!currentProject) return;
      if (share) {
        const result = await shareProject({
          projectId: currentProject.id,
          workspaceId: currentProject.workspaceId,
        });
        if (result.sharingUrl) {
          const token = result.sharingUrl.split("/shared/").pop();
          if (token) {
            yMetadata?.set("sharingToken", token);
          }
        }
      } else {
        yMetadata?.set("sharingToken", null);
        const result = await unshareProject({
          projectId: currentProject.id,
          workspaceId: currentProject.workspaceId,
        });
        if (!result.projectId) {
          // Revert on failure
          yMetadata?.set("sharingToken", currentProject.sharedToken ?? null);
        }
      }
    },
    [currentProject, shareProject, unshareProject, yMetadata],
  );

  return {
    sharingUrl,
    handleProjectShare,
  };
};
