import { useCallback, useEffect, useMemo } from "react";
import { useY } from "react-yjs";
import type { Doc } from "yjs";
import { Map as YMap } from "yjs";

import { useSharedProject } from "@flow/lib/gql";
import type { Project, YDocMetadataValue } from "@flow/types";

const emptyMetadata = new YMap<YDocMetadataValue>();

export default ({
  currentProject,
  yDoc,
}: {
  currentProject?: Project | undefined;
  yDoc: Doc | null;
}) => {
  const { shareProject, unshareProject } = useSharedProject();

  const yMetadata = useMemo(
    () => yDoc?.getMap<YDocMetadataValue>("metadata"),
    [yDoc],
  );
  const metadata = useY(yMetadata ?? emptyMetadata);

  // undefined until the project has loaded, null when it isn't shared.
  const serverSharingToken = currentProject
    ? (currentProject.sharedToken ?? null)
    : undefined;

  // Keep ydoc in sync with server. Every set is a document update, so write
  // only when the doc disagrees; an unconditional write would change the
  // document each time the project is opened. A missing key falls back to the
  // server value below, so it counts as agreeing with an unshared project.
  useEffect(() => {
    if (!yMetadata || serverSharingToken === undefined) return;
    if ((yMetadata.get("sharingToken") ?? null) === serverSharingToken) return;
    yMetadata.set("sharingToken", serverSharingToken);
  }, [yMetadata, serverSharingToken]);

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
