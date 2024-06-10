import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { useWorkspace } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";

export const useCheckWorkspace = (workspaceId: string) => {
  const [_, setCurrentWorkspace] = useCurrentWorkspace();
  const navigate = useNavigate();

  const { getWorkspaces } = useWorkspace();
  const { workspaces, isLoading } = getWorkspaces();

  useEffect(() => {
    if (!workspaces) return;
    const selectedWorkspace = workspaces?.find(w => w.id === workspaceId);

    if (!selectedWorkspace) {
      const route = window.location.pathname;
      // TODO: This returns a promise but it can't be awaited
      navigate({ to: route.replace(workspaceId, workspaces[0].id), replace: true });
      return;
    }

    setCurrentWorkspace(selectedWorkspace);
  }, [workspaces, navigate, setCurrentWorkspace, workspaceId]);

  return { workspaces, isLoading };
};
