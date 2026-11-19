import { useCallback, useEffect, useState } from "react";

import { useProject, useSharedProject } from "@flow/lib/gql";
import type { Project } from "@flow/types";

// The sharing token lives on the Project, not in the shared document, so
// sharing never counts as a workflow edit.
export default ({ currentProject }: { currentProject?: Project }) => {
  const { shareProject, unshareProject } = useSharedProject();
  const { refetchProject } = useProject();

  const projectToken = currentProject?.sharedToken ?? null;

  // The token this user just set (null when unsharing), shown until the
  // project is fetched again and agrees.
  const [requestedToken, setRequestedToken] = useState<string | null>();

  useEffect(() => {
    if (requestedToken !== undefined && requestedToken === projectToken) {
      setRequestedToken(undefined);
    }
  }, [requestedToken, projectToken]);

  const sharingToken =
    requestedToken !== undefined ? requestedToken : projectToken;

  const sharingUrl = sharingToken
    ? `${window.location.origin}/shared/${sharingToken}`
    : undefined;

  const projectId = currentProject?.id;

  // Another member may have shared or unshared since the project was fetched.
  const handleProjectShareOpen = useCallback(() => {
    if (projectId) refetchProject(projectId);
  }, [projectId, refetchProject]);

  const handleProjectShare = useCallback(
    async (share: boolean) => {
      if (!currentProject) return;
      const input = {
        projectId: currentProject.id,
        workspaceId: currentProject.workspaceId,
      };
      if (share) {
        const result = await shareProject(input);
        const token = result.sharingUrl?.split("/shared/").pop();
        if (token) setRequestedToken(token);
      } else {
        setRequestedToken(null);
        const result = await unshareProject(input);
        if (!result.projectId) setRequestedToken(undefined);
      }
    },
    [currentProject, shareProject, unshareProject],
  );

  return {
    sharingUrl,
    handleProjectShare,
    handleProjectShareOpen,
  };
};
