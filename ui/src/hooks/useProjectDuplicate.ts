import { useCallback, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useAuth } from "@flow/lib/auth";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Project, ProjectDocument } from "@flow/types";

export default (project: Project, projectDocument?: ProjectDocument) => {
  const { getAccessToken } = useAuth();
  const t = useT();
  const [isDuplicating, setIsDuplicating] = useState<boolean>(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const { createProject } = useProject();

  const handleProjectDuplication = useCallback(async () => {
    if (!project || !currentWorkspace) {
      return;
    }
    const { websocket } = config();

    try {
      setIsDuplicating(true);

      const { project: newProject } = await createProject({
        workspaceId: currentWorkspace.id,
        name: project.name + t("(duplicate)"),
        description: project.description,
      });

      if (!projectDocument || !newProject) {
        setIsDuplicating(false);
        return;
      }

      const updates = projectDocument?.updates;
      if (!updates || !updates.length) {
        setIsDuplicating(false);
        return;
      }

      const yDoc = new Y.Doc();
      const convertedUpdates = new Uint8Array(updates);

      if (websocket) {
        const token = await getAccessToken();
        const yWebSocketProvider = new WebsocketProvider(
          websocket,
          `${newProject.id}:${DEFAULT_ENTRY_GRAPH_ID}`,
          yDoc,
          { params: { token } },
        );
        try {
          await new Promise<void>((resolve) => {
            yWebSocketProvider.once("sync", () => {
              yDoc.transact(() => {
                Y.applyUpdate(yDoc, convertedUpdates);
              });

              setIsDuplicating(false);
              resolve();
            });
          });
        } finally {
          yWebSocketProvider?.destroy();
        }
      }
    } catch (error) {
      console.error("Project duplication failed:", error);
      setIsDuplicating(false);
    }
  }, [
    currentWorkspace,
    t,
    getAccessToken,
    createProject,
    project,
    projectDocument,
  ]);

  return {
    isDuplicating,
    handleProjectDuplication,
  };
};
